import { randomUUID } from 'crypto';
import fs from 'fs';
import path from 'path';

const SUPABASE_URL = 'http://127.0.0.1:8000';
let SUPABASE_ANON_KEY = '';
try {
    const envFile = fs.readFileSync(path.join(process.cwd(), '../apps/dashboard/.env.local'), 'utf-8');
    const match = envFile.match(/NEXT_PUBLIC_SUPABASE_ANON_KEY=(.+)/);
    if (match) SUPABASE_ANON_KEY = match[1].trim();
} catch (e) {}

async function fetchApi(path, options = {}) {
    const res = await fetch(`${SUPABASE_URL}${path}`, {
        ...options,
        headers: {
            'Content-Type': 'application/json',
            'apikey': SUPABASE_ANON_KEY,
            ...options.headers,
        }
    });
    const text = await res.text();
    let json;
    try { json = JSON.parse(text); } catch(e) { json = text; }
    if (res.status >= 400) {
        console.log(`Error ${res.status} on ${path}:`, json);
    }
    return { status: res.status, data: json };
}

async function signUp(email, password) {
    const res = await fetchApi('/auth/v1/signup', {
        method: 'POST',
        body: JSON.stringify({ email, password })
    });
    return res;
}

const TABLES = ['devices', 'activity_sessions', 'settings', 'tombstones'];

function getPayload(table, id, deviceId) {
    if (table === 'devices') return { id, device_name: 'Test Device', platform: 'windows', first_registered_at: new Date().toISOString(), settings_mode: 'local' };
    if (table === 'activity_sessions') return { id, device_id: deviceId, source: 'windows', application_name: 'test.exe', title: 'Test', started_at: new Date().toISOString(), ended_at: new Date().toISOString(), duration_ms: 5000, created_at: new Date().toISOString() };
    if (table === 'settings') return { id, device_id: deviceId, version: 1, mode: 'local', payload: { "key": "val" }, updated_at: new Date().toISOString() };
    if (table === 'tombstones') return { id, device_id: deviceId, target_record_id: randomUUID(), deletion_timestamp: new Date().toISOString(), schema_version: 1, sync_state: 'synced', created_at: new Date().toISOString() };
}

async function run() {
    const ts = Date.now();
    const userA = await signUp(`userA_${ts}@example.com`, 'password123');
    const userB = await signUp(`userB_${ts}@example.com`, 'password123');
    
    const userA_id = userA.data.user.id;
    const userB_id = userB.data.user.id;
    const tokenA = userA.data.access_token;
    const tokenB = userB.data.access_token;

    console.log("==================================================");
    console.log("4. VERIFY auth.uid() DEFAULTS");
    console.log("==================================================");
    for (const table of TABLES) {
        console.log(`\n--- TABLE: ${table} ---`);
        const id = randomUUID();
        const deviceId = table === 'devices' ? id : randomUUID(); // Use actual device_id if needed, but for FK constraints devices must exist. Wait, devices table must exist first!
        // Always seed devices for user A and user B to satisfy foreign keys!
        
        let validDeviceIdA = randomUUID();
        let validDeviceIdB = randomUUID();
        if (table !== 'devices') {
            await fetchApi('/rest/v1/devices', {
                method: 'POST',
                headers: { 'Authorization': `Bearer ${tokenA}` },
                body: JSON.stringify({ id: validDeviceIdA, device_name: 'A', platform: 'win', first_registered_at: new Date().toISOString(), settings_mode: 'local' })
            });
            await fetchApi('/rest/v1/devices', {
                method: 'POST',
                headers: { 'Authorization': `Bearer ${tokenB}` },
                body: JSON.stringify({ id: validDeviceIdB, device_name: 'B', platform: 'win', first_registered_at: new Date().toISOString(), settings_mode: 'local' })
            });
        }
        const useDeviceIdA = table === 'devices' ? id : validDeviceIdA;

        // INSERT without user_id
        let payload1 = getPayload(table, id, useDeviceIdA);
        const res1 = await fetchApi(`/rest/v1/${table}`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' },
            body: JSON.stringify(payload1)
        });
        const passed1 = res1.status === 201 && res1.data[0].user_id === userA_id;
        console.log(`Authenticated User A: INSERT without user_id -> ${passed1 ? "PASS" : "FAIL"} (Status: ${res1.status}, ID: ${res1.data[0]?.user_id})`);

        // INSERT claiming User B
        const id2 = randomUUID();
        let payload2 = getPayload(table, id2, useDeviceIdA);
        payload2.user_id = userB_id;
        const res2 = await fetchApi(`/rest/v1/${table}`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' },
            body: JSON.stringify(payload2)
        });
        // Postgres RLS blocks inserting rows where user_id != auth.uid() when using "auth.uid() = user_id" with CHECK constraint.
        // It returns 403 or similar if it violates the CHECK clause, OR 201 but 0 rows if it violates something else. Actually standard RLS WITH CHECK policy returns 403 "new row violates row-level security policy for table".
        const passed2 = res2.status === 403 || res2.status === 401 || (res2.status === 201 && res2.data.length === 0);
        console.log(`Authenticated User A: INSERT explicitly claiming User B -> ${passed2 ? "PASS" : "FAIL"} (Status: ${res2.status})`);

        // Unauthenticated request
        const id3 = randomUUID();
        let payload3 = getPayload(table, id3, useDeviceIdA);
        const res3 = await fetchApi(`/rest/v1/${table}`, {
            method: 'POST',
            headers: { 'Prefer': 'return=representation' },
            body: JSON.stringify(payload3)
        });
        const passed3 = res3.status === 401 || res3.status === 403 || (res3.status === 201 && res3.data.length === 0);
        console.log(`Unauthenticated request: INSERT without user_id -> ${passed3 ? "PASS" : "FAIL"} (Status: ${res3.status})`);
    }

    console.log("\n==================================================");
    console.log("5. COMPLETE RLS CRUD MATRIX");
    console.log("==================================================");
    for (const table of TABLES) {
        console.log(`\n--- TABLE: ${table} ---`);
        let validDeviceIdA = randomUUID();
        let validDeviceIdB = randomUUID();
        
        // Seed devices for foreign keys
        await fetchApi('/rest/v1/devices', { method: 'POST', headers: { 'Authorization': `Bearer ${tokenA}` }, body: JSON.stringify({ id: validDeviceIdA, device_name: 'A', platform: 'win', first_registered_at: new Date().toISOString(), settings_mode: 'local' }) });
        await fetchApi('/rest/v1/devices', { method: 'POST', headers: { 'Authorization': `Bearer ${tokenB}` }, body: JSON.stringify({ id: validDeviceIdB, device_name: 'B', platform: 'win', first_registered_at: new Date().toISOString(), settings_mode: 'local' }) });

        const idA = randomUUID();
        const idB = randomUUID();
        const useDeviceIdA = table === 'devices' ? idA : validDeviceIdA;
        const useDeviceIdB = table === 'devices' ? idB : validDeviceIdB;

        // CREATE User A Record
        await fetchApi(`/rest/v1/${table}`, {
            method: 'POST', headers: { 'Authorization': `Bearer ${tokenA}` }, body: JSON.stringify(getPayload(table, idA, useDeviceIdA))
        });
        // CREATE User B Record
        await fetchApi(`/rest/v1/${table}`, {
            method: 'POST', headers: { 'Authorization': `Bearer ${tokenB}` }, body: JSON.stringify(getPayload(table, idB, useDeviceIdB))
        });

        // User A SELECT User B
        const selectB_A = await fetchApi(`/rest/v1/${table}?id=eq.${idB}`, { method: 'GET', headers: { 'Authorization': `Bearer ${tokenA}` } });
        console.log(`User A SELECT User B -> ${selectB_A.data.length === 0 ? "PASS" : "FAIL"} (Status: ${selectB_A.status}, Rows: ${selectB_A.data.length})`);

        // User A INSERT claiming User B -> same as above
        const idB_fake = randomUUID();
        let payloadClaimingB = getPayload(table, idB_fake, useDeviceIdB);
        payloadClaimingB.user_id = userB_id;
        const insertB_A = await fetchApi(`/rest/v1/${table}`, { method: 'POST', headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' }, body: JSON.stringify(payloadClaimingB) });
        console.log(`User A INSERT claiming User B -> ${[401, 403].includes(insertB_A.status) || (insertB_A.status === 201 && insertB_A.data.length === 0) ? "PASS" : "FAIL"} (Status: ${insertB_A.status})`);

        const patchPayload = table === 'devices' ? { device_name: 'hacked' } : table === 'activity_sessions' ? { duration_ms: 0 } : table === 'settings' ? { mode: 'hacked' } : { sync_state: 'hacked' };

        // User A UPDATE User B
        const updateB_A = await fetchApi(`/rest/v1/${table}?id=eq.${idB}`, { method: 'PATCH', headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' }, body: JSON.stringify(patchPayload) });
        console.log(`User A UPDATE User B -> ${(updateB_A.data && updateB_A.data.length === 0) ? "PASS" : "FAIL"} (Status: ${updateB_A.status}, Rows: ${updateB_A.data ? updateB_A.data.length : 0})`);

        // User A DELETE User B
        const deleteB_A = await fetchApi(`/rest/v1/${table}?id=eq.${idB}`, { method: 'DELETE', headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' } });
        console.log(`User A DELETE User B -> ${(deleteB_A.data && deleteB_A.data.length === 0) ? "PASS" : "FAIL"} (Status: ${deleteB_A.status}, Rows: ${deleteB_A.data ? deleteB_A.data.length : 0})`);

        // User B SELECT User A
        const selectA_B = await fetchApi(`/rest/v1/${table}?id=eq.${idA}`, { method: 'GET', headers: { 'Authorization': `Bearer ${tokenB}` } });
        console.log(`User B SELECT User A -> ${selectA_B.data.length === 0 ? "PASS" : "FAIL"} (Status: ${selectA_B.status}, Rows: ${selectA_B.data.length})`);

        // User B INSERT claiming User A
        let payloadClaimingA = getPayload(table, randomUUID(), useDeviceIdA);
        payloadClaimingA.user_id = userA_id;
        const insertA_B = await fetchApi(`/rest/v1/${table}`, { method: 'POST', headers: { 'Authorization': `Bearer ${tokenB}`, 'Prefer': 'return=representation' }, body: JSON.stringify(payloadClaimingA) });
        console.log(`User B INSERT claiming User A -> ${[401, 403].includes(insertA_B.status) || (insertA_B.status === 201 && insertA_B.data.length === 0) ? "PASS" : "FAIL"} (Status: ${insertA_B.status})`);

        // User B UPDATE User A
        const updateA_B = await fetchApi(`/rest/v1/${table}?id=eq.${idA}`, { method: 'PATCH', headers: { 'Authorization': `Bearer ${tokenB}`, 'Prefer': 'return=representation' }, body: JSON.stringify(patchPayload) });
        console.log(`User B UPDATE User A -> ${(updateA_B.data && updateA_B.data.length === 0) ? "PASS" : "FAIL"} (Status: ${updateA_B.status}, Rows: ${updateA_B.data ? updateA_B.data.length : 0})`);

        // User B DELETE User A
        const deleteA_B = await fetchApi(`/rest/v1/${table}?id=eq.${idA}`, { method: 'DELETE', headers: { 'Authorization': `Bearer ${tokenB}`, 'Prefer': 'return=representation' } });
        console.log(`User B DELETE User A -> ${(deleteA_B.data && deleteA_B.data.length === 0) ? "PASS" : "FAIL"} (Status: ${deleteA_B.status}, Rows: ${deleteA_B.data ? deleteA_B.data.length : 0})`);
    }
}

run().catch(console.error);
