use term::buffer::{Buff};

fn gen_lines(len: usize) -> Vec<String> {
  let mut lines = vec![];

  for i in 0..len {
    lines.push(format!("Line{}", i))
  }

  lines
}

#[test]
fn append_with_empty_blocks_pushes_data_to_the_new_block() {
  let mut buff = Buff::new(vec![], 32);
  let new_content = gen_lines(20);

  buff.append(new_content.clone());

  assert_eq!(buff.content(), [new_content].to_vec())
}

#[test]
fn append_generates_new_blocks_to_fit_data() {
  let mut buff = Buff::new(["Foo".to_string()].to_vec(), 2);
  let new_content = gen_lines(4);

  buff.append(new_content.clone());

  assert_eq!(
    buff.content(),
    [
      [
        "Foo".to_string(),
        "Line0".to_string(),
      ].to_vec(),
      [
        "Line1".to_string(),
        "Line2".to_string(),
      ].to_vec(),
      [
        "Line3".to_string(),
      ].to_vec(),
    ].to_vec()
  )
}
