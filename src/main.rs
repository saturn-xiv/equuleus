#[macro_use]
extern crate pug;
extern crate equuleus;

fn main() {
    if let Err(e) = pug::log4rs::init_file("log4rs.yml", Default::default()) {
        pug::env_logger::init();
        error!("failed to parse log4rs.yml, {:?}", e);
    }
    if let Err(e) = equuleus::app::launch() {
        error!("{:?}", e);
    }
}
