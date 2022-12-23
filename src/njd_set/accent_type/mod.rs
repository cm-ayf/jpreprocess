mod rule;

use crate::njd::pos::*;
use crate::njd::*;

pub fn njd_set_accent_type(njd: &mut NJD) {
    if njd.nodes.len() == 0 {
        return;
    }
    let mut top_node_i: Option<usize> = None;
    let mut mora_size: i32 = 0;
    for i in 0..njd.nodes.len() {
        let mut top_node_acc: Option<i32> = None;
        let mut prev_acc: Option<i32> = None;
        let mut current_acc: Option<i32> = None;

        {
            let (top_node, prev, current, next) = if i == 0 {
                (None, None, njd.nodes.get(0).unwrap(), njd.nodes.get(1))
            } else {
                let top_node = top_node_i.and_then(|i| njd.nodes.get(i));
                (
                    top_node,
                    njd.nodes.get(i - 1),
                    njd.nodes.get(i).unwrap(),
                    njd.nodes.get(i + 1),
                )
            };

            if i == 0 || !matches!(current.get_chain_flag(), Some(d) if d) {
                top_node_i = Some(i);
                mora_size = 0;
            } else if prev.is_some() && matches!(current.get_chain_flag(), Some(d) if d) {
                top_node_acc = Some(calc_top_node_acc(
                    current,
                    prev.as_ref().unwrap(),
                    top_node.as_ref().unwrap(),
                    mora_size,
                ));
            }

            if matches!(current.get_chain_flag(), Some(true))
                && matches!(prev.map(|n| n.get_pos().get_group1()), Some(Group1::Kazu))
                && matches!(current.get_pos().get_group1(), Group1::Kazu)
            {
                prev_acc = calc_digit_acc(prev.unwrap(), current, next);
            }

            if current.get_string() == rule::JYUU
                && !matches!(current.get_chain_flag(), Some(d) if d)
                && matches!(next.map(|n| n.get_pos().get_group1()), Some(Group1::Kazu))
            {
                current_acc = Some(0);
            }

            mora_size += current.get_mora_size();
        }

        if let (Some(top_node_i), Some(top_node_acc)) = (top_node_i, top_node_acc) {
            njd.nodes.get_mut(top_node_i).unwrap().set_acc(top_node_acc);
        }
        if let Some(prev_acc) = prev_acc {
            njd.nodes.get_mut(i - 1).unwrap().set_acc(prev_acc);
        }
        if let Some(current_acc) = current_acc {
            njd.nodes.get_mut(i).unwrap().set_acc(current_acc);
        }
    }
}

fn calc_top_node_acc(node: &NJDNode, prev: &NJDNode, top_node: &NJDNode, mora_size: i32) -> i32 {
    let (add_type, rule) = get_rule(node.get_chain_rule(), prev.get_pos());
    let node_acc = node.get_acc();
    let top_node_acc = top_node.get_acc();
    match rule.as_str() {
        "F1" => top_node_acc,
        "F2" if top_node_acc == 0 => mora_size + add_type,
        "F3" if top_node_acc != 0 => mora_size + add_type,
        "F4" => mora_size + add_type,
        "F5" => 0,
        "C1" => mora_size + node_acc,
        "C2" => mora_size + 1,
        "C3" => mora_size,
        "C4" => 0,
        "C5" => top_node_acc,
        "P1" => {
            if top_node_acc == 0 {
                0
            } else {
                mora_size + node_acc
            }
        }
        "P2" => {
            if top_node_acc == 0 {
                mora_size + 1
            } else {
                mora_size + node_acc
            }
        }
        "P6" => 0,
        "P14" if top_node_acc != 0 => mora_size + node_acc,
        _ => top_node_acc,
    }
}

fn get_rule(input_rule_option: Option<&str>, prev_pos: &PartOfSpeech) -> (i32, String) {
    if let Some(input_rule) = input_rule_option {
        'segs: for seg in input_rule.split("/") {
            enum BufferContent {
                Rule,
                AddType,
            }

            let mut buf = String::new();
            let mut buf_content: BufferContent = BufferContent::Rule;

            let mut rule = String::new();
            let mut add_type = 0;

            for c in seg.chars() {
                match c {
                    '%' => {
                        if !prev_pos.group0_contains(buf.as_str()) {
                            continue 'segs;
                        }
                        buf = String::new();
                        buf_content = BufferContent::Rule;
                    }
                    '@' => {
                        if matches!(buf_content, BufferContent::Rule) {
                            rule = buf;
                        }
                        buf = String::new();
                        buf_content = BufferContent::AddType;
                    }
                    _ => buf.push(c),
                }
            }
            match buf_content {
                BufferContent::Rule => {
                    rule = buf;
                }
                BufferContent::AddType => {
                    add_type = buf.parse().unwrap();
                }
            }
            return (add_type, rule);
        }
    }
    (0, "*".to_string())
}

fn calc_digit_acc(prev: &NJDNode, current: &NJDNode, next: Option<&NJDNode>) -> Option<i32> {
    let prev_str = prev.get_string();
    let current_str = current.get_string();
    let next_str = next.map(|node| node.get_string());
    match (prev_str, current_str, next_str) {
        (
            rule::GO | rule::ROKU | rule::HACHI,
            rule::JYUU,
            Some(
                rule::ICHI
                | rule::NI
                | rule::SAN
                | rule::YON
                | rule::GO
                | rule::ROKU
                | rule::NANA
                | rule::HACHI
                | rule::KYUU,
            ),
        ) => Some(0),
        // (rule::SAN | rule::YON | rule::KYUU | rule::NAN | rule::SUU, rule::JYUU, _) => Some(1),
        (_, rule::JYUU, _) => Some(1),

        (rule::NANA, rule::HYAKU, _) => Some(2),
        (rule::SAN | rule::YON | rule::KYUU | rule::NAN, rule::HYAKU, _) => Some(1),
        (_, rule::HYAKU, _) => Some(prev.get_mora_size() + current.get_mora_size()),

        (_, rule::SEN, _) => Some(prev.get_mora_size() + 1),

        (_, rule::MAN, _) => Some(prev.get_mora_size() + 1),

        (rule::ICHI | rule::ROKU | rule::NANA | rule::HACHI | rule::IKU, rule::OKU, _) => Some(2),
        (_, rule::OKU, _) => Some(1),

        (rule::ROKU | rule::NANA, rule::CHOU, _) => Some(2),
        (_, rule::CHOU, _) => Some(1),

        _ => None,
    }
}
