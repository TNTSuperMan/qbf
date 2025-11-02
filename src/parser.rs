pub enum BFNode {
    Add(u8),
    Set(u8),
    To(isize),
    Out,
    In,
    LoopStart(usize),
    LoopEnd(usize, bool), // is_flat
}

pub fn parse(code: &str) -> Vec<BFNode> {
    let mut nodes: Vec<BFNode> = Vec::new();
    let mut loop_stack: Vec<usize> = Vec::new();
    let mut last_loop_start: usize = 0;

    for char in code.chars() {
        match char {
            '+' => {
                if let Some(BFNode::Add(value)) = nodes.last_mut() {
                    *value = value.wrapping_add(1);
                } else if let Some(BFNode::Set(value)) = nodes.last_mut() {
                    *value = value.wrapping_add(1);
                } else {
                    nodes.push(BFNode::Add(1));
                }
            }
            '-' => {
                if let Some(BFNode::Add(value)) = nodes.last_mut() {
                    *value = value.wrapping_sub(1);
                } else if let Some(BFNode::Set(value)) = nodes.last_mut() {
                    *value = value.wrapping_sub(1);
                } else {
                    nodes.push(BFNode::Add(255)); // 255u8 = -1i8
                }
            }
            '>' => {
                if let Some(BFNode::To(to)) = nodes.last_mut() {
                    *to += 1;
                } else {
                    nodes.push(BFNode::To(1));
                }
            }
            '<' => {
                if let Some(BFNode::To(to)) = nodes.last_mut() {
                    *to -= 1;
                } else {
                    nodes.push(BFNode::To(-1));
                }
            }
            '.' => {
                nodes.push(BFNode::Out);
            }
            ',' => {
                nodes.push(BFNode::In);
            }
            '[' => {
                let start = nodes.len();
                last_loop_start = start;
                loop_stack.push(start); // ループ先頭のASTポインタになるよ
                nodes.push(BFNode::LoopStart(usize::MAX));
            }
            ']' => {
                let start = loop_stack.pop().unwrap();
                let end = nodes.len(); // 上のコメントと同じ感じ
                if end - start == 2 {
                    match nodes.last() {
                        Some(BFNode::Add(255)) => {
                            nodes.truncate(nodes.len() - 2);
                            nodes.push(BFNode::Set(0));
                            continue;
                        }
                        _ => {}
                    }
                }
                nodes.push(BFNode::LoopEnd(start, last_loop_start == start));
                nodes[start] = BFNode::LoopStart(end);
            }
            _ => {}
        }
    }

    nodes
}
