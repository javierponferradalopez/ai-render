use mermaid_rs_renderer::parse_mermaid;

pub struct Counts {
    pub nodes: usize,
    pub edges: usize,
}

pub fn count(source: &str) -> Result<Counts, String> {
    let parsed = parse_mermaid(source).map_err(|error| error.to_string())?;
    Ok(Counts {
        nodes: parsed.graph.nodes.len(),
        edges: parsed.graph.edges.len(),
    })
}
