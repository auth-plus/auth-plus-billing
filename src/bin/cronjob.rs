use auth_plus_billing::presentation;
fn main() -> std::io::Result<()> {
    presentation::cronjob::start()
}
