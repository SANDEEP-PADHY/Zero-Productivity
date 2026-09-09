pub fn normalize_app_name(name: &str) -> String {
    let mut n = name.trim().to_lowercase();
    if n.ends_with(".exe") {
        n.truncate(n.len() - 4);
    }
    n
}

pub fn normalize_app_path(path: &str) -> String {
    path.trim().to_lowercase().replace('\\', "/")
}

pub fn generate_app_id(name: &str, path: Option<&str>) -> String {
    let norm_name = normalize_app_name(name);
    if let Some(p) = path {
        let norm_path = normalize_app_path(p);
        format!("app:{}:{}", norm_name, norm_path)
    } else {
        format!("app:{}", norm_name)
    }
}

pub fn normalize_domain(domain: &str) -> String {
    let mut d = domain.trim().to_lowercase();
    if d.starts_with("www.") {
        d = d[4..].to_string();
    }
    if d.ends_with('.') {
        d.pop();
    }
    if let Some(idx) = d.find(':') {
        d.truncate(idx);
    }
    d
}

pub fn normalize_url(url: &str) -> String {
    let mut u = url.trim().to_string();
    if let Some(idx) = u.find('#') {
        u.truncate(idx);
    }
    if let Some(idx) = u.find("://") {
        let prefix_len = idx + 3;
        let rest = &u[prefix_len..];
        if let Some(slash_idx) = rest.find('/') {
            let host_part = rest[..slash_idx].to_lowercase();
            let path_part = &rest[slash_idx..];
            format!("{}{}{}", u[..prefix_len].to_lowercase(), host_part, path_part)
        } else {
            u.to_lowercase()
        }
    } else {
        u
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_app_name() {
        assert_eq!(normalize_app_name("Code.exe"), "code");
        assert_eq!(normalize_app_name("  Notepad.EXE  "), "notepad");
        assert_eq!(normalize_app_name("slack"), "slack");
    }

    #[test]
    fn test_normalize_app_path() {
        assert_eq!(normalize_app_path("C:\\Program Files\\App\\App.exe"), "c:/program files/app/app.exe");
        assert_eq!(normalize_app_path(" /usr/bin/app  "), "/usr/bin/app");
    }

    #[test]
    fn test_generate_app_id() {
        assert_eq!(generate_app_id("Code.exe", None), "app:code");
        assert_eq!(generate_app_id("Code", Some("C:\\App\\Code.exe")), "app:code:c:/app/code.exe");
    }

    #[test]
    fn test_normalize_domain() {
        assert_eq!(normalize_domain("www.Example.com"), "example.com");
        assert_eq!(normalize_domain("  github.com.  "), "github.com");
        assert_eq!(normalize_domain("localhost:8080"), "localhost");
    }

    #[test]
    fn test_normalize_url() {
        assert_eq!(normalize_url("HTTP://WWW.EXAMPLE.COM/Path?Q=1#fragment"), "http://www.example.com/Path?Q=1");
        assert_eq!(normalize_url("https://github.com"), "https://github.com");
        assert_eq!(normalize_url("just_a_string#hash"), "just_a_string");
    }
}
