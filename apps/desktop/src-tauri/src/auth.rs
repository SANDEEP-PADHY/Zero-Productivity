pub mod api {
    use keyring::Entry;
    use std::sync::Mutex;
    use tauri::State;

    pub struct AuthState {
        pub service_name: String,
    }

    impl AuthState {
        pub fn new() -> Self {
            Self {
                service_name: "zero_productivity_tracker".to_string(),
            }
        }
    }

    impl Default for AuthState {
        fn default() -> Self {
            Self::new()
        }
    }

    impl AuthState {
        pub fn get_access_token(&self) -> Result<String, String> {
            if let Ok(token) = std::env::var("TEST_ACCESS_TOKEN") {
                return Ok(token);
            }
            let entry = Entry::new(&self.service_name, "access_token")
                .map_err(|e| e.to_string())?;
            entry.get_password().map_err(|e| e.to_string())
        }

        pub fn set_access_token(&self, token: &str) -> Result<(), String> {
            let entry = Entry::new(&self.service_name, "access_token")
                .map_err(|e| e.to_string())?;
            entry.set_password(token).map_err(|e| e.to_string())
        }

        pub fn get_refresh_token(&self) -> Result<String, String> {
            let entry = Entry::new(&self.service_name, "refresh_token")
                .map_err(|e| e.to_string())?;
            entry.get_password().map_err(|e| e.to_string())
        }

        pub fn set_refresh_token(&self, token: &str) -> Result<(), String> {
            let entry = Entry::new(&self.service_name, "refresh_token")
                .map_err(|e| e.to_string())?;
            entry.set_password(token).map_err(|e| e.to_string())
        }

        pub fn logout(&self) -> Result<(), String> {
            let access_entry = Entry::new(&self.service_name, "access_token")
                .map_err(|e| e.to_string())?;
            let _ = access_entry.delete_credential();

            let refresh_entry = Entry::new(&self.service_name, "refresh_token")
                .map_err(|e| e.to_string())?;
            let _ = refresh_entry.delete_credential();

            Ok(())
        }
    }

    #[tauri::command]
    pub fn login(
        auth_state: State<'_, Mutex<AuthState>>,
        access_token: String,
        refresh_token: String,
    ) -> Result<(), String> {
        let state = auth_state.lock().map_err(|_| "Mutex poisoned")?;
        state.set_access_token(&access_token)?;
        state.set_refresh_token(&refresh_token)?;
        Ok(())
    }

    #[tauri::command]
    pub fn logout(auth_state: State<'_, Mutex<AuthState>>) -> Result<(), String> {
        let state = auth_state.lock().map_err(|_| "Mutex poisoned")?;
        state.logout()?;
        Ok(())
    }
}
