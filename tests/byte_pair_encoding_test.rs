use std::io::ErrorKind;

use tokenizers_rs::BytePairEncoding;

const TEXT: &str = "This is not a token.";

#[test]
fn bpe_tokenizes_text() {
    let tokenizer = BytePairEncoding::from(TEXT.to_string(), 18);

    let expected = vec![
        "<|startoftext|>".to_string(),
        "T".to_string(),
        "h".to_string(),
        "is".to_string(),
        " ".to_string(),
        "token".to_string(),
        " ".to_string(),
        "is".to_string(),
        " ".to_string(),
        "n".to_string(),
        "ot".to_string(),
        "<|endoftext|>".to_string(),
    ];
    let actual = tokenizer.tokenize("This token is not".to_string());

    assert!(actual.is_ok());
    assert_eq!(expected, actual.unwrap());
}

#[test]
fn bpe_throws_error_for_unseen_word() {
    let tokenizer = BytePairEncoding::from(TEXT.to_string(), 18);

    let res = tokenizer.tokenize("This token is not real".to_string());

    assert!(res.is_err());
}

#[test]
fn bpe_throws_io_error() {
    let tokenizer = BytePairEncoding::from(TEXT.to_string(), 18);

    let expected_err_kind = ErrorKind::InvalidInput;
    let expected_err_msg = "Word not found in vocabulary";
    let actal = tokenizer
        .tokenize("This token is not real".to_string())
        .err()
        .unwrap();

    assert_eq!(expected_err_kind, actal.kind());
    assert_eq!(expected_err_msg, actal.to_string());
}
