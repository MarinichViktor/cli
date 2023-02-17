const DEFAULT_BUFFER_BLOCK_SIZE: i32 = 100;

pub struct Buff {
  pub blocks: Vec<Block>,
  offset: (usize,i32),
  block_size: i32
}

impl Default for Buff {
  fn default() -> Self {
    Self {
      blocks: vec![],
      offset: (0,0),
      block_size: DEFAULT_BUFFER_BLOCK_SIZE
    }
  }
}

impl Buff {
  pub fn append(&mut self,  mut buff: Vec<String>) {
    let mut block: &mut Block = match self.blocks.last_mut() {
      Some(b) => b,
      None => {
        self.blocks.push(self.new_block());
        self.blocks.last_mut().unwrap()
      }
    };

    loop {
      let avail_capacity = block.cap();
      let (data, remain) = {
        if buff.len() as i32 > avail_capacity {
          let res = buff.split_at(avail_capacity as usize);
          (res.0.to_vec(), res.1.to_vec())
        } else {
          (buff, vec![])
        }
      };

      block.append(data);
      buff = remain.to_vec();

      if buff.is_empty() {
        break;
      }

      self.blocks.push(self.new_block());

      block = self.blocks.last_mut().unwrap();
    }
  }

  pub fn content(&self) -> Vec<Vec<String>> {
    let mut container = vec![];

    for block in &self.blocks {
      container.push(block.content.clone())
    }

    container
  }

  pub fn new(content: Vec<String>, offset: (usize,i32), block_size: i32) -> Self {
    let mut buff = Self {
      blocks: vec![],
      offset: offset,
      block_size: block_size
    };
    buff.append(content);
    buff
  }

  fn new_block(&self) -> Block {
    Block::new(self.block_size)
  }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
  content: Vec<String>,
  size: i32
}

impl Block {
  pub fn new(size: i32) -> Self {
    Self {
      content: vec![],
      size
    }
  }

  pub fn len(&self) -> usize {
    self.content.len()
  }

  pub fn is_empty(&self) -> bool {
    self.content.is_empty()
  }

  pub fn cap(&self) -> i32 {
    self.size - self.content.len() as i32
  }

  pub fn append(&mut self, mut data: Vec<String>) {
    if data.len() as i32 > self.cap() {
      panic!("There are not enough capacity to append data");
    }

    self.content.append(&mut data);
  }

  pub fn data(&self) -> &[String] {
    &self.content
  }
}

#[cfg(test)]
mod block_tests {
  use super::{Block};

  fn gen_lines(len: usize) -> Vec<String> {
    let mut lines = vec![];

    for i in 0..len {
      lines.push(format!("Line{}", i))
    }

    lines
  }

  #[test]
  fn len_returns_content_length() {
    let block = Block {
      content: gen_lines(3),
      size: 10
    };

    assert_eq!(block.len(), 3);
  }

  #[test]
  fn is_empty_with_empty_block_returns_true() {
    let block = Block {
      content: vec![],
      size: 10
    };

    assert_eq!(block.is_empty(), true);
  }

  #[test]
  fn is_empty_with_non_empty_block_returns_false() {
    let block = Block {
      content: gen_lines(3),
      size: 10
    };

    assert_eq!(block.is_empty(), false);
  }

  #[test]
  fn cap_returns_available_space_size() {
    let block = Block {
      content: gen_lines(3),
      size: 10
    };

    assert_eq!(block.cap(), 7);
  }

  #[test]
  fn cap_returns_block_capacity() {
    let block = Block {
      content: gen_lines(3),
      size: 10
    };

    assert_eq!(block.cap(), 7);
  }

  #[test]
  fn append_with_enough_capcity_adds_data_to_block() {
    let intial_content = gen_lines(3);
    let mut block = Block {
      content: intial_content.to_vec(),
      size: 5
    };

    block.append(["Foo".to_string(), "Bar".to_string()].to_vec());

    let mut expecred_result = gen_lines(3);
    expecred_result.push("Foo".to_string());
    expecred_result.push( "Bar".to_string());

    assert_eq!(block.content, expecred_result);
  }

  #[test]
  #[should_panic(expected = "There are not enough capacity to append data")]
  fn append_without_enough_capcity_panics() {
    let intial_content = gen_lines(3);
    let mut block = Block {
      content: intial_content.to_vec(),
      size: 4
    };

    block.append(["Foo".to_string(), "Bar".to_string()].to_vec());
  }
}