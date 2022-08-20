pub fn hello_from_lib() {
    println!("Hello from lib!");
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        assert(true);
        println!("It works!");
    }
}
