pub const BUILD_INFO: &str = build_info::format!(
    "{} v{} by {}",
    $.crate_info.name,
    $.crate_info.version,
    $.crate_info.authors,
);

pub const APP_NAME: &str = build_info::format!("{}", $.crate_info.name);
