#[macro_use]
extern crate log;
extern crate env_logger;
extern crate equuleus;
extern crate log4rs;

fn main() {
    if let Err(e) = log4rs::init_file("log4rs.yml", Default::default()) {
        env_logger::init();
        error!("failed to parse log4rs.yml, {:?}", e);
    }
    println!("Hello, world!");
}
