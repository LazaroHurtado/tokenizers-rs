use tokenizers_rs::BytePairEncoding;

#[test]
fn it_works() {
    let expected = vec![
        "<|startoftext|>".to_string(),
        "This".to_string(),
        " ".to_string(),
        "is".to_string(),
        " not".to_string(),
        " ".to_string(),
        "a".to_string(),
        " token".to_string(),
        ".".to_string(),
        "<|endoftext|>".to_string(),
    ];
    let actual = BytePairEncoding::tokenize("This is not a token.".to_string(), 25);

    assert_eq!(expected, actual);
}
