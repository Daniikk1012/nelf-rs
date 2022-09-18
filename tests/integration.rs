use nelf::{NelfIter, ToNelf};

#[test]
fn test_1() {
    let fourth = [b"1", b"2", b"3"];
    let list = [b"A".to_vec(), b"B".to_vec(), b"C".to_vec(), fourth.to_nelf()];
    let nelf = list.clone().to_nelf();
    let result: Vec<_> = NelfIter::from_string(&nelf).collect();
    assert_eq!(result, list);
    let result: Vec<_> = NelfIter::from_string(result[3]).collect();
    assert_eq!(result, fourth);
}

#[test]
fn test_2() {
    let item = b"ABCD/|\\";
    let nelf = [[[item].to_nelf()].to_nelf()].to_nelf();
    assert_eq!(
        NelfIter::from_string(
            NelfIter::from_string(NelfIter::from_string(&nelf).next().unwrap())
                .next()
                .unwrap(),
        )
        .next()
        .unwrap(),
        item,
    );
}
