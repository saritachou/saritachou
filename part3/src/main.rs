mod graph_utils; // Import local modules 
mod customer;
use std::error::Error;
use crate::customer::{Customer, OneHotEncoding, print_top_shared_characteristics};
use graph_utils::{construct_graph, calculate_centrality, identify_high_centrality_nodes};
use crate::customer::map_category;



pub fn main() -> Result<(), Box<dyn Error>> {
    // Read the CSV file and create a vector of Customer structs
    let mut rdr = csv::Reader::from_path("BankChurners.csv")?;
    let customers: Vec<Customer> = rdr
        .records()
        .take(1000)
        .map(|result| {
            let record = result?; // unwrap result to get the record
            Ok::<Customer, Box<dyn Error>>(Customer {
                // extract values from record
                churn_status: record.get(1).unwrap_or(&"Unknown".to_string()).to_string(),
                age: record[1].parse().unwrap_or(2),
                one_hot_encoding: OneHotEncoding {
                    education_level: map_category(record.get(5).unwrap_or(&"Unknown".to_string())),
                    marital_status: map_category(record.get(6).unwrap_or(&"Unknown".to_string())),
                    income_range: map_category(record.get(7).unwrap_or(&"Unknown".to_string())),
                    card_type: map_category(record.get(8).unwrap_or(&"Unknown".to_string())),
                },
                mon_w_bank: record[9].parse().unwrap_or(0),
                num_product_purchased: record[10].parse().unwrap_or(0),
                mon_inactive: record[11].parse().unwrap_or(0),
                num_contact: record[12].parse().unwrap_or(0),
                //card_credit_limit: record[13].parse().unwrap_or(0),
                //evolving_bal: record[14].parse().unwrap_or(0),
                transactions_amount: record[17].parse().unwrap_or(0),
                num_transctions: record[18].parse().unwrap_or(0),
                avg_card_utilize: record[20].parse().unwrap_or(0.0),
            })
        })
        .collect::<Result<_, _>>()?;

    let graph = construct_graph(&customers);

    // Splitting customers into two groups: churned customers and customers who haven't churned (churn=stop using card)
    let (not_churn_customers, churn_customers): (Vec<_>, Vec<_>) =
        customers.iter().cloned().partition(|customer| customer.churn_status == "Existing Customer");

    let churn_centrality = calculate_centrality(&graph, &churn_customers); // Calculate centrality for churned customers
    let not_churn_centrality = calculate_centrality(&graph, &not_churn_customers);  // Calculate centrality for not churned customers

    let churn_high_centrality_nodes = identify_high_centrality_nodes(&churn_centrality, 1.1); // Identify high centrality nodes for churned customers
    let not_churn_high_centrality_nodes = identify_high_centrality_nodes(&not_churn_centrality, 1.1);// Identify high centrality nodes for churned customers

    // Print high centrality nodes for churned customers and the top 4 shared characteristics between those nodes and their neighbors 
    println!("Churn High Centrality Nodes");
    print_top_shared_characteristics(&churn_high_centrality_nodes, &churn_customers, &graph);
    // Print high centrality nodes for not churned customers and the top 4 shared characteristics between those nodes and their neighbors 
    println!("Not Churn High Centrality Nodes:");
    print_top_shared_characteristics(&not_churn_high_centrality_nodes, &not_churn_customers, &graph);

    Ok(())
}


