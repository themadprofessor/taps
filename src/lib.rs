pub mod connection;
pub mod error;
pub mod preconnection;
pub mod properties;
mod race;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
