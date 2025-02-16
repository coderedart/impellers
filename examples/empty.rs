fn main() {
    // just to check that linking worked
    let version = unsafe { impellers::sys::ImpellerGetVersion() };
    dbg!(version);
}
