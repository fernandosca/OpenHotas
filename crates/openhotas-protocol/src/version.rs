/// Protocol version — validated by firmware before accepting configuration.
/// If `protocol_version_major` doesn't match, the firmware refuses the config.
pub const PROTOCOL_VERSION_MAJOR: u8 = 3;
pub const PROTOCOL_VERSION_MINOR: u8 = 1;
