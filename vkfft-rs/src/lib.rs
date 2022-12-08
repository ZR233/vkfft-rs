extern crate vkfft_src as vk;
pub mod config;
pub mod app;
pub mod error;
mod version;
pub use version::*;



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let version = version();


        println!("version: {:?}", version);

    }
}
