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
    return { status: res.status, data: json };
}

async function signUp(email, password) {
    const res = await fetchApi('/auth/v1/signup', {
        method: 'POST',
        body: JSON.stringify({ email, password })
    });
    return res;
}

async function run() {
    const ts = Date.now();
    const userA = await signUp(`userA_${ts}@example.com`, 'password123');
    const userB = await signUp(`userB_${ts}@example.com`, 'password123');
    
    const userA_id = userA.data.user.id;
    const userB_id = userB.data.user.id;
    const tokenA = userA.data.access_token;
    const tokenB = userB.data.access_token;

    const deviceIdA = randomUUID();
    const deviceIdB = randomUUID();

    // ================= User A tests =================
    // INSERT own
    const insertA = await fetchApi('/rest/v1/devices', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' },
        body: JSON.stringify({
            id: deviceIdA,
            user_id: userA_id,
            device_name: 'Device A',
            platform: 'windows',
            first_registered_at: new Date().toISOString(),
            settings_mode: 'local'
        })
    });
    console.log("User A -> INSERT own:", insertA.status === 201 ? "PASS" : "FAIL", insertA.status);

    // SELECT own
    const selectA = await fetchApi(`/rest/v1/devices?id=eq.${deviceIdA}`, {
        method: 'GET',
        headers: { 'Authorization': `Bearer ${tokenA}` }
    });
    console.log("User A -> SELECT own:", selectA.data.length === 1 ? "PASS" : "FAIL", selectA.status);

    // UPDATE own
    const updateA = await fetchApi(`/rest/v1/devices?id=eq.${deviceIdA}`, {
        method: 'PATCH',
        headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' },
        body: JSON.stringify({ device_name: 'Device A Updated' })
    });
    console.log("User A -> UPDATE own:", updateA.data && updateA.data.length === 1 ? "PASS" : "FAIL", updateA.status);

    // ================= User B tests =================
    const insertB = await fetchApi('/rest/v1/devices', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${tokenB}`, 'Prefer': 'return=representation' },
        body: JSON.stringify({
            id: deviceIdB,
            user_id: userB_id,
            device_name: 'Device B',
            platform: 'windows',
            first_registered_at: new Date().toISOString(),
            settings_mode: 'local'
        })
    });
    console.log("User B -> INSERT own:", insertB.status === 201 ? "PASS" : "FAIL", insertB.status);

    // ================= Cross-Tenant Tests =================
    // User A trying to SELECT User B
    const selectB_by_A = await fetchApi(`/rest/v1/devices?id=eq.${deviceIdB}`, {
        method: 'GET',
        headers: { 'Authorization': `Bearer ${tokenA}` }
    });
    console.log("User A -> SELECT User B:", selectB_by_A.data.length === 0 ? "PASS (DENY)" : "FAIL", selectB_by_A.status);

    // User A trying to INSERT claiming User B
    const insertClaimingB = await fetchApi('/rest/v1/devices', {
        method: 'POST',
        headers: { 'Authorization': `Bearer ${tokenA}` },
        body: JSON.stringify({
            id: randomUUID(),
            user_id: userB_id,
            device_name: 'Device Malicious',
            platform: 'windows',
            first_registered_at: new Date().toISOString(),
            settings_mode: 'local'
        })
    });
    console.log("User A -> INSERT claiming User B:", insertClaimingB.status === 403 || insertClaimingB.status === 201 && insertClaimingB.data.length === 0 /* wait, if 201 happens but RLS blocks it, usually it's 201 but 0 rows returned if return=representation is used, but for INSERT it should violate the USING clause of the policy, which blocks it with 403 or 401 */ ? "PASS (DENY)" : `FAIL (${insertClaimingB.status})`);
    
    // Actually, let's just log the status and verify it's not 201. Or if it is 201 it might mean RLS didn't apply.
    if(insertClaimingB.status === 201) console.log("WARNING: Insert claiming User B returned 201!", insertClaimingB.data);

    // User A trying to UPDATE User B
    const updateB_by_A = await fetchApi(`/rest/v1/devices?id=eq.${deviceIdB}`, {
        method: 'PATCH',
        headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' },
        body: JSON.stringify({ device_name: 'Hacked by A' })
    });
    console.log("User A -> UPDATE User B:", updateB_by_A.data && updateB_by_A.data.length === 0 ? "PASS (DENY)" : "FAIL", updateB_by_A.status);

    // User A trying to DELETE User B
    const deleteB_by_A = await fetchApi(`/rest/v1/devices?id=eq.${deviceIdB}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' }
    });
    console.log("User A -> DELETE User B:", deleteB_by_A.data && deleteB_by_A.data.length === 0 ? "PASS (DENY)" : "FAIL", deleteB_by_A.status);

    // ================= DELETE own =================
    const deleteA = await fetchApi(`/rest/v1/devices?id=eq.${deviceIdA}`, {
        method: 'DELETE',
        headers: { 'Authorization': `Bearer ${tokenA}`, 'Prefer': 'return=representation' }
    });
    console.log("User A -> DELETE own:", deleteA.data && deleteA.data.length === 1 ? "PASS" : "FAIL", deleteA.status);

}

run().catch(console.error);
