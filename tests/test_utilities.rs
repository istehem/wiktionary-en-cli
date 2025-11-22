#[cfg(test)]
mod tests {
    use rstest::rstest;
    use utilities::colored_string_utils::{format_integer, Join};

    #[rstest]
    fn joining_strings() -> () {
        let vec: Vec<String> = [1, 2, 3, 4, 5].iter().map(|&num| num.to_string()).collect();
        let joined = String::from(",").join(vec);
        assert_eq!(joined, "1,2,3,4,5");
    }

    #[rstest]
    fn joining_a_singelton_list() -> () {
        let vec: Vec<String> = [1].iter().map(|&num| num.to_string()).collect();
        let joined = String::from(",").join(vec);
        assert_eq!(joined, "1");
    }

    #[rstest]
    fn formatting_zero() -> () {
        assert_eq!(format_integer(0).to_ascii_lowercase(), "0");
    }

    #[rstest]
    fn formatting_million() -> () {
        assert_eq!(format_integer(1000000).to_ascii_lowercase(), "1,000,000");
    }

    #[rstest]
    fn formatting_ten_thousand() -> () {
        assert_eq!(format_integer(10000).to_ascii_lowercase(), "10,000");
    }
}
