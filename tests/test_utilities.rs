#[cfg(test)]
mod tests {
    use rstest::*;
    use utilities::colored_string_utils::Join;

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
}
