
pub struct App {
    pub content: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            content: "".to_string()
        }
    }
}

pub enum FormattedLines {
    Cached(Vec<String>),
    Unprocessed
}

impl App {
    pub fn lines(&mut self, width: u16) -> Vec<String> {
        self.content.lines()
          .flat_map(|line| {
            let mut curr = line;
            let mut sublines = vec![];

            while curr.len() > width as usize {
                let (a,b) = curr.split_at(width as usize);
                sublines.push(a);
                curr = b;
            }

            if !curr.is_empty() {
                sublines.push(curr);
            }

            sublines
        })
          .map(str::to_owned)
          .collect()
    }
}

fn split_by_size<'a>(input: &'a str, len: u16) -> Vec<&'a str> {
    input.lines().map(|line| {
        let mut curr = line;
        let mut sublines = vec![];

        while curr.len() > len as usize {
            let (a,b) = curr.split_at(len as usize);
            sublines.push(a);
            curr = b;
        }

        if !curr.is_empty() {
            sublines.push(curr);
        }

        sublines
    })
      .flatten()
      .collect()
}