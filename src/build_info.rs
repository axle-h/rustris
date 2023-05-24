pub const BUILD_INFO: &str = build_info::format!(
    "{} v{}-{} by {}",
    $.crate_info.name,
    $.crate_info.version,
    $.version_control.unwrap().git().unwrap().commit_short_id,
    $.crate_info.authors,
);
