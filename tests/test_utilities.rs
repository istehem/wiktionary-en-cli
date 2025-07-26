#[cfg(test)]
mod tests {
    use rstest::*;
    use utilities::colored_string_utils::{format_integer, Join};

    #[rstest]
    fn test_join_strings() -> () {
        let vec: Vec<String> = [1, 2, 3, 4, 5].iter().map(|&num| num.to_string()).collect();
        let joined = String::from(",").join(vec);
        assert_eq!(joined, "1,2,3,4,5");
    }

    #[rstest]
    fn test_join_a_singelton_list() -> () {
        let vec: Vec<String> = [1].iter().map(|&num| num.to_string()).collect();
        let joined = String::from(",").join(vec);
        assert_eq!(joined, "1");
    }

    #[rstest]
    fn test_format_zero() -> () {
        assert_eq!(format_integer(0).to_ascii_lowercase(), "0");
    }

    #[rstest]
    fn test_format_million() -> () {
        assert_eq!(format_integer(1000000).to_ascii_lowercase(), "1,000,000");
    }

    #[rstest]
    fn test_format_ten_thousand() -> () {
        assert_eq!(format_integer(10000).to_ascii_lowercase(), "10,000");
    }
}
