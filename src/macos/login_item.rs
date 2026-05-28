#[cfg(target_os = "macos")]
pub fn set_launch_at_login(_enabled: bool) -> Result<(), String> {
    Err(
        "Launch-at-login registration is pending ServiceManagement integration in the Rust port"
            .into(),
    )
}

#[cfg(not(target_os = "macos"))]
pub fn set_launch_at_login(_enabled: bool) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn non_macos_stub_is_safe() {
        #[cfg(not(target_os = "macos"))]
        super::set_launch_at_login(false).unwrap();
    }
}
