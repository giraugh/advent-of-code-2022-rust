/* Util Structs */

mod grid;

/* Importing */

#[macro_export]
macro_rules! aoc_input {
    () => {
        aoc_input!("./input.txt")
    };
    ($path:expr) => {{
        let arg = std::env::args().skip(1).next();
        let path = arg.unwrap_or(($path).to_string());
        std::fs::read_to_string((&path))
            .unwrap_or_else(|_| panic!("Couldn't find AOC input file: {}", &path))
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(&aoc_input!(), "hello world!\n");
        assert_eq!(&aoc_input!("./input.txt"), "hello world!\n");
    }
}
