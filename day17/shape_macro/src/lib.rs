use proc_macro::TokenStream;
use proc_macro::TokenTree;

#[derive(Debug, PartialEq, Eq)]
enum ShapeElement {
    Fill,
    Space,
    NewLine,
}

#[proc_macro]
pub fn shape(_item: TokenStream) -> TokenStream {
    // Parse stream
    let shape_elements = _item
        .into_iter()
        .map(|token_tree| match token_tree {
            TokenTree::Punct(punct) => match punct.as_char() {
                '@' => ShapeElement::Fill,
                ',' => ShapeElement::NewLine,
                '.' => ShapeElement::Space,
                _ => panic!("Unknown character"),
            },
            _ => panic!("Unexpected token"),
        })
        .collect::<Vec<_>>();

    // Split elements into lines
    let shape_lines = shape_elements
        .split(|el| *el == ShapeElement::NewLine)
        .collect::<Vec<_>>();

    let shape_offsets = shape_lines
        .iter()
        .enumerate()
        .flat_map(|(y, &line)| {
            line.iter()
                .enumerate()
                .filter(|(_, cell)| **cell == ShapeElement::Fill)
                .map(move |(x, _)| (x, y))
        })
        .collect::<Vec<_>>();

    let textual = format!("vec!{:?}", shape_offsets);
    textual.parse().unwrap()
}
