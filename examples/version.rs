fn main() {
    let header_version = impellers::ImpellerVersion::get_header_version();
    let linked_version = impellers::ImpellerVersion::get_linked_version();
    println!(
        "header version: {}.{}.{}",
        header_version.get_major(),
        header_version.get_minor(),
        header_version.get_patch()
    );
    println!(
        "linked version: {}.{}.{}",
        linked_version.get_major(),
        linked_version.get_minor(),
        linked_version.get_patch()
    );
}
