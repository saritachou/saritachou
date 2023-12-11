use petgraph::graph::{Graph, NodeIndex};
use petgraph::algo::dijkstra;
use std::collections::HashMap;
use petgraph::Undirected;
use crate::customer::{Customer}; // Import the Customer struct from the local module

// Function to construct a graph from customers
pub fn construct_graph(customers: &[Customer]) -> Graph<&Customer, (), Undirected> {
    let mut graph = Graph::new_undirected(); // Create an undirected graph
    let node_indices: Vec<NodeIndex> = customers.iter().map(|customer| graph.add_node(customer)).collect();

    // Iterate through pairs of customers and add edges if conditions are met
    for (i, &customer_a) in node_indices.iter().enumerate() {
        for (j, &customer_b) in node_indices.iter().enumerate() {
            if i != j && determine_neighbor(graph.node_weight(customer_a).unwrap(), graph.node_weight(customer_b).unwrap()) { // use helper function determine_neighbor to check condition
                graph.add_edge(customer_a, customer_b, ()); // Add an edge between customers with shared characteristics
            }
        }
    }

    graph// Return the constructed graph
}

// Function to determine if two customers (=nodes) are neighbors (base on wehther the number of share characteristics is above threshold)
//helper function used in construct_graph
pub fn determine_neighbor(customer_a: &Customer, customer_b: &Customer) -> bool {
    let mut shared_characteristics_count = 0; // Initialize a count for number of shared characteristics between two nodes 

    let is_similar = |value_a: &str, value_b: &str| value_a == value_b; // Closure to check if two values are similar
    let in_same_group = |value_a: &str, value_b: &str, groups: &[&str]| { // Closure to check if two values are in the same group
        groups.iter().any(|&group| value_a == group && value_b == group)
    };
    if is_similar(&customer_a.age.to_string(), &customer_b.age.to_string()) { // Check and increment count for shared characteristics for age
        shared_characteristics_count += 1;
    }
    if is_similar(&customer_a.one_hot_encoding.education_level, &customer_b.one_hot_encoding.education_level) { // Check and increment count for shared characteristics for education level
        shared_characteristics_count += 1;
    }
    if is_similar(&customer_a.one_hot_encoding.marital_status, &customer_b.one_hot_encoding.marital_status) { // Check and increment count for shared characteristics for marital status
        shared_characteristics_count += 1;
    }
    if is_similar(&customer_a.one_hot_encoding.income_range, &customer_b.one_hot_encoding.income_range) { // Check and increment count for shared characteristics for income range
        shared_characteristics_count += 1;
    }
    if is_similar(&customer_a.one_hot_encoding.card_type, &customer_b.one_hot_encoding.card_type) {// Check and increment count for shared characteristics for card type
        shared_characteristics_count += 1;
    }
    if in_same_group(&customer_a.mon_w_bank.to_string(), &customer_b.mon_w_bank.to_string(), &vec!["20-30", "30-40", "40-50", ">50"]) {// Check and increment count for shared characteristics for months with the bank
        shared_characteristics_count += 1;
    }
    
    if is_similar(&customer_a.num_product_purchased.to_string(), &customer_b.num_product_purchased.to_string()) {
        shared_characteristics_count += 1;
    }
    
    if is_similar(&customer_a.mon_inactive.to_string(), &customer_b.mon_inactive.to_string()) {
        shared_characteristics_count += 1;
    }

    if is_similar(&customer_a.num_contact.to_string(), &customer_b.num_contact.to_string()) {
        shared_characteristics_count += 1;
    }
    if in_same_group(&customer_a.transactions_amount.to_string(), &customer_b.transactions_amount.to_string(), &vec!["500<", "500-1000", "1000-1500","1500-2000",">2000"]) {
        shared_characteristics_count += 1;
    }
    if in_same_group(&customer_a.num_transctions.to_string(), &customer_b.num_transctions.to_string(), &vec!["<10","10-20","20-30","30-40",">40"]) {
        shared_characteristics_count += 1;
    }
    
    if in_same_group(&customer_a.avg_card_utilize.to_string(), &customer_b.avg_card_utilize.to_string(), &vec!["<0.100","0.100-0.200","0.200-0.300","0.300-0.400",">0.400"]) {
        shared_characteristics_count += 1;
    }
    // Adjust the threshold as needed; if the number of shared characteristic is above this threshold, we connect the two customers
    shared_characteristics_count >= 2
}

// Function to calculate centrality for each node in the graph

pub fn calculate_centrality(graph: &Graph<&Customer, (), Undirected>, customers: &[Customer]) -> HashMap<NodeIndex, f64> {
    let petgraph_indices: Vec<NodeIndex> = customers.iter().enumerate().map(|(i, _)| NodeIndex::new(i)).collect(); // Create node indices for customers
    let mut all_distances: HashMap<NodeIndex, HashMap<NodeIndex, f64>> = HashMap::new();// HashMap to store distances between nodes
    for node in &petgraph_indices {
        let mut distances: HashMap<NodeIndex, f64> = HashMap::new();
        for nodew in &petgraph_indices {
            if node != nodew {
                if let Some(distance) = all_distances.get(nodew).and_then(|map| map.get(node)) {
                    distances.insert(*nodew, *distance);
                } else {
                    let distance_map = dijkstra(graph, *node, Some(*nodew), |_edge| 1.0);
                    let distance = *distance_map.get(&nodew).unwrap_or(&f64::INFINITY);
                    distances.insert(*nodew, distance);
                }
            }
        }
        all_distances.insert(*node, distances);
    }
     //calculate centrality of each nodes using normalized closeness centrality
     let centrality: HashMap<_, _> = petgraph_indices.iter().map(|&node| {
        let distance_sum: f64 = all_distances[&node].values().copied().sum();
        let centrality = (customers.len() - 1) as f64 / distance_sum;
        (node, centrality)
    }).collect();

    centrality // Return the HashMap of node indices and their centrality values
}

// Function to identify nodes with high centrality
pub fn identify_high_centrality_nodes(centrality: &HashMap<NodeIndex, f64>, threshold_factor: f64) -> Vec<NodeIndex> {
    let threshold = threshold_factor * centrality.values().sum::<f64>() / centrality.len() as f64; // Adjusted threshold
    centrality.iter().filter_map(|(&node, &centrality)| {
        if centrality > threshold {// Return the node index if its centrality is above the threshold
            Some(node)
        } else {
            None
        }
    }).collect()// Return a vector of node indices with high centrality
}
