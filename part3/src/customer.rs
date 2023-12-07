use petgraph::graph::{Graph, NodeIndex};
use petgraph::Undirected;
use crate::graph_utils::determine_neighbor;

// create a struct for catergorical variables' one-hot encoding 
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OneHotEncoding {
    pub education_level: String,
    pub marital_status: String,
    pub income_range: String,
    pub card_type: String,
}

// define a customer struct with 11 attributes (attributes = categories of characteristics we want to analyze) 
// ex: age is a category, age groups customers belong to are characteristics
#[allow(dead_code)]
#[derive(PartialEq)]
#[derive(Clone, Debug)]
pub struct Customer {
    pub churn_status: String, // whether the customer is still using the card (not churn) or not (churn)
    pub age: i32, //age of customer
    pub one_hot_encoding: OneHotEncoding, // see struct OneHotEncoding
    pub mon_w_bank: i32, // number of months the customer has been using services/purchasing products from the bank
    pub num_product_purchased: i32, // number of products the customer purchased from the bank
    pub mon_inactive: i32, // number of months the customer's card is inactive
    pub num_contact: i32, // number of times the banks contacted the customer
    pub card_credit_limit: i32, // card's credit limit in dollars 
    pub evolving_bal: i32, // evolving balance of card (available balance left) in dollars
    pub transactions_amount: i32, // dollar amount of card transactions 
    pub num_transctions: i32, // number of card transactions in the pat 12 months
    pub avg_card_utilize: f64, // Average Card Utilization Ratio (divide your balance by your credit limit)
}

// Function to print the top 4 shared characteristics between high centrality nodes and their neighbors
pub fn print_top_shared_characteristics( 
    high_centrality_nodes: &[NodeIndex], // slice of NodeIndex representing high centrality nodes
    customers: &[Customer],//Slice of Customer representing all customers
    graph: &Graph<&Customer, (), Undirected>, // Reference to the undirected graph of customers (constructed in graph_utils and passed in in main)
) -> Result<(), Box<dyn std::error::Error>> {
    if high_centrality_nodes.is_empty() { // print statement in case there is no high centrality nodes
        println!("No high centrality nodes.");
        return Ok(());
    }
    // Create a HashMap to store each category's total counts and separated counts by characteristics in each category
    let mut total_characteristic_counts = std::collections::HashMap::<String, usize>::new();
    let mut separated_counts: std::collections::HashMap<String, std::collections::HashMap<String, usize>> =
        std::collections::HashMap::new();
    
    // iterate over high centrality ndoes 
    for &node_index in high_centrality_nodes { 
        if node_index.index() < customers.len() { // Check if the node index is within the bounds of the customers array
            let shared_characteristics =
                find_top_shared_characteristics(graph, node_index, customers); // Find the top 4 shared characteristics between the current node and its neighbors using helper function

            if shared_characteristics.is_empty() {// Continue to the next iteration if there are no shared characteristics
                continue;
            }

            // Update the total counts for each characteristic across nodes 
            for (characteristic, count) in shared_characteristics.iter().cloned() {
                *total_characteristic_counts.entry(characteristic.clone()).or_insert(0) += count;

                // sort characteristics into the categories they belong to 
                // do this by splitting the characteristic names by ":", the string before is category lable, after is characteristic
                let parts: Vec<&str> = characteristic.splitn(2, ":").collect(); 
                if parts.len() == 2 {
                    let key = parts[0].trim().to_string();
                    let entry_count = separated_counts
                        .entry(key.clone())
                        .or_insert_with(|| std::collections::HashMap::new());

                    *entry_count.entry(parts[1].trim().to_string()).or_insert(0) += count;
                }
            }
        } else { // print statement for invalid node index
            println!("Invalid node index: {}", node_index.index());
        }
    }

    // Calculate the sum of total counts of shared characteristic across all categories (for percentage calculation later)
    let total_sum: usize = separated_counts.iter().flat_map(|(_, entry_counts)| entry_counts.values()).sum();

    // Print the total counts for each categories and the characteristics within each category
    println!("Prevalent characteristic categories and their compositions:");
    for (key, entry_counts) in separated_counts.iter() { // iterate through each category and their characteristics 
        let total_count: usize = entry_counts.values().cloned().sum(); // calculate total characteritics count of each category
        let key_percentage: f64 = (total_count as f64 / total_sum as f64) * 100.00; // calculate the percentage of each category 
        let rounded_key_percentage = (key_percentage * 10.0).round() / 10.0; // round the percentage 

        println!("{}, (Total Count: {} - {}%)", key, total_count, rounded_key_percentage); //print the name, total count and percentage of each category 

        for (entry, count) in entry_counts.iter() { // iterate through each characteristics and their counts 
            let percentage: f64 = (count.clone() as f64 / total_count as f64) * 100.00; // calculate the percentage of each characteristic within their category
            let rounded_percentage = (percentage * 10.0).round() / 10.0; //  round the percentage 
            println!("  {}: {} ({}%)", entry, count, rounded_percentage); // print the name, total count and percentage of each characteristic
        }
    }
    println!("");

    Ok(())
}

//Function to find the top 4 shared characteristics between a given node and its neighbors
// helper function used in print_top_shared_characteristics
pub fn find_top_shared_characteristics(
    graph: &Graph<&Customer, (), Undirected>, // Reference to the undirected graph of customers
    node_index: NodeIndex, // Node index for a specific customer
    customers: &[Customer],// Slice of Customer representing all customers
) -> Vec<(String, usize)> { // Vector of tuples containing top shared characteristics and their counts (counts=number of time they are shared between a centrality node and its neighbor)
    let mut characteristic_counts = std::collections::HashMap::<String, usize>::new(); // Create a HashMap to store characteristic counts

    for neighbor_index in graph.neighbors(node_index) { // Iterate over neighbors of the given node
        // Check if the neighbor index is within the bounds of the customers array
        if neighbor_index.index() < customers.len() { 
            let neighbor = &customers[neighbor_index.index()];
            // Get the shared characteristics between the node and the current neighbor using helper function get_shared_characteristics
            let shared_characteristics = get_shared_characteristics( 
                &customers[node_index.index()],
                neighbor,
            );

            for characteristic in shared_characteristics {  
                // Update the count for the shared characteristic
                *characteristic_counts.entry(characteristic).or_insert(0) += 1;
            }
        }
    }
    // Create a sorted vector of characteristic counts
    let mut sorted_characteristics: Vec<_> = characteristic_counts.into_iter().collect();
    sorted_characteristics.sort_by(|(_, count1), (_, count2)| count2.cmp(count1));
    // Return the top 4 shared characteristics
    sorted_characteristics.into_iter().take(4).collect()
}   

// Function to get shared characteristics between two nodes (nodes=customers)
pub fn get_shared_characteristics(customer_a: &Customer, customer_b: &Customer) -> Vec<String> {
    let mut shared_characteristics: Vec<String> = Vec::new(); // Create a vector to store shared characteristics
    let is_similar = |value_a: &str, value_b: &str| value_a == value_b; // Closure to check if two values are similar
    let in_same_group = |value_a: &str, value_b: &str, groups: &[&str]| { // Closure to check if two values are in the same group
        groups.iter().any(|&group| value_a == group && value_b == group)
    };
    if is_similar(&customer_a.age.to_string(), &customer_b.age.to_string()) {// Check and add shared characteristics for age
        shared_characteristics.push(format!("Age: {}", customer_a.age));
    }
    if is_similar(&customer_a.one_hot_encoding.education_level, &customer_b.one_hot_encoding.education_level) { // Check and add shared characteristics for education level
        shared_characteristics.push(format!("Education Level: {}", &customer_a.one_hot_encoding.education_level));
    }
    if is_similar(&customer_a.one_hot_encoding.marital_status, &customer_b.one_hot_encoding.marital_status) {// Check and add shared characteristics for marital status
        shared_characteristics.push(format!("Marital Status: {}", &customer_a.one_hot_encoding.marital_status));
    }
    if is_similar(&customer_a.one_hot_encoding.income_range, &customer_b.one_hot_encoding.income_range) {// Check and add shared characteristics for income range
        shared_characteristics.push(format!("Income Range: {}", &customer_a.one_hot_encoding.income_range));
    }
    if is_similar(&customer_a.one_hot_encoding.card_type, &customer_b.one_hot_encoding.card_type) { // Check and add shared characteristics for card type
        shared_characteristics.push(format!("Card Type: {}", &customer_a.one_hot_encoding.card_type));
    }
    // Check and add shared characteristics for Mon W Bank
    if in_same_group(&customer_a.mon_w_bank.to_string(), &customer_b.mon_w_bank.to_string(), &vec!["20-30", "30-40", "40-50", ">50"]) { // create groups and compare whether two nodes are in the same group
        shared_characteristics.push(format!("Mon W Bank: {}", &customer_a.mon_w_bank));
    }
    if is_similar(&customer_a.num_product_purchased.to_string(), &customer_b.num_product_purchased.to_string()) { // Check and add shared characteristics for the number of products
        shared_characteristics.push(format!("Number of Products Purchased: {}", customer_a.num_product_purchased));
    }
    if is_similar(&customer_a.mon_inactive.to_string(), &customer_b.mon_inactive.to_string()) {// Check and add shared characteristics for the month inactive
        shared_characteristics.push(format!("Month inactive: {}", customer_a.mon_inactive));
    }
    if is_similar(&customer_a.num_contact.to_string(), &customer_b.num_contact.to_string()) {// Check and add shared characteristics for the number of contacts from the bank
        shared_characteristics.push(format!("Number of Contacts from Bank (past 12 months): {}", customer_a.num_contact));
    }
    // Check and add shared characteristics for card credit limit; create groups and compare whether two nodes are in the same group
    if in_same_group(&customer_a.card_credit_limit.to_string(), &customer_b.card_credit_limit.to_string(), &vec!["5000<", "5000-10000", "10000-15000", "15000-20000","20000-25000","25000-30000",">30000"]) {
        shared_characteristics.push(format!("Card's Credit Limit: {}", &customer_a.card_credit_limit));
    }
    // Check and add shared characteristics for evolving balance on card; create groups and compare whether two nodes are in the same group
    if in_same_group(&customer_a.evolving_bal.to_string(), &customer_b.evolving_bal.to_string(), &vec!["500<", "500-1000", "1000-1500","1500-2000",">2000"]) {
        shared_characteristics.push(format!("Evolving Balance on Card: {}", &customer_a.evolving_bal));
    }
    // Check and add shared characteristics for total dollar amount of transactions via card骯create groups and compare whether two nodes are in the same group
    if in_same_group(&customer_a.transactions_amount.to_string(), &customer_b.transactions_amount.to_string(), &vec!["500<", "500-1000", "1000-1500","1500-2000",">2000"]) {
        shared_characteristics.push(format!("Total Dollar Amount of Transaction via Card: {}", &customer_a.transactions_amount));
    }
    // Check and add shared characteristics for total number of transactions via card; create groups and compare whether two nodes are in the same group
    if in_same_group(&customer_a.num_transctions.to_string(), &customer_b.num_transctions.to_string(), &vec!["<10","10-20","20-30","30-40",">40"]) {
        shared_characteristics.push(format!("Total Number of Transactions via Card: {}", &customer_a.num_transctions));
    }
    // Check and add shared characteristics for average card utilization ratio; create groups and compare whether two nodes are in the same group
    if in_same_group(&customer_a.avg_card_utilize.to_string(), &customer_b.avg_card_utilize.to_string(), &vec!["<0.100","0.100-0.200","0.200-0.300","0.300-0.400",">0.400"]) {
        shared_characteristics.push(format!("Average Card Utilization Ratio: {}", &customer_a.avg_card_utilize));
    }

    shared_characteristics // // Return the vector of shared characteristics
}

// Function to map categorical values
pub fn map_category(value: &str) -> String {
    match value {
        "Unknown" => "Unknown".to_string(),// Unknown category
        "High School" | "Graduate" | "Uneducated" | "College" | "Post-Graduate" | "Doctorate" => value.to_string(),  // Education categories
        "Married" | "Single" | "Divorced" => value.to_string(),// Marital status categories
        "Less than $40K" | "$40K - $60K" | "$60K - $80K" | "$80K - $120K" | "$120K +" => value.to_string(),// Income range categories 
        "Blue" | "Silver" | "Gold" | "Platinum" => value.to_string(), // Card type categories
        _ => "Unknown".to_string(), // Default to unknown category
    }
}

#[cfg(test)]

pub mod tests {
    use super::*;

    // test whether the get_shared_characteristics function is working correctly
    #[test]
    pub fn test_shared_characteristics() {
        // Create two customers with known characteristics
        let customer1 = create_sample_customer1();
        let customer2 = create_sample_customer2();

        // Use the get_shared_characteristics function to find shared characteristics
        let shared_characteristics = get_shared_characteristics(&customer1, &customer2);
        let correct_shared_characteristics =  ["Education Level: Graduate", "Marital Status: Single", "Income Range: $40K - $60K", "Card Type: Silver"];
        // Verify that the shared characteristics are correct
        assert_eq!(shared_characteristics, correct_shared_characteristics);
    }
    #[test]
    pub fn test_determine_neighbor(){
        // Create two customers with known characteristics
        let customer1 = create_sample_customer1();
        let customer2 = create_sample_customer2();
        // Use the determine_neighbor function to see whether the two customers fit the condition to be neighbors (in the undirected graph)
        let test_neighbor = determine_neighbor(&customer1, &customer2);
        let correct_neighbor = true;
        // Verify that determine_neighbor correctly determines the two customers are neighbors  
        assert_eq!(test_neighbor, correct_neighbor);
    }

    //pub fn determine_neighbor(customer_a: &Customer, customer_b: &Customer) -> bool {

    // Helper functions to create two sample customers with known characteristics
    pub fn create_sample_customer1() -> Customer {
        Customer {
            churn_status: "Existing Customer".to_string(),
            age: 25,
            one_hot_encoding: OneHotEncoding {
                education_level: "Graduate".to_string(),
                marital_status: "Single".to_string(),
                income_range: "$40K - $60K".to_string(),
                card_type: "Silver".to_string(),
            },
            mon_w_bank: 12,
            num_product_purchased: 5,
            mon_inactive: 2,
            num_contact: 8,
            card_credit_limit: 15000,
            evolving_bal: 1200,
            transactions_amount: 5000,
            num_transctions: 25,
            avg_card_utilize: 0.4,
        }
    }
    
    pub fn create_sample_customer2() -> Customer {
        Customer {
            churn_status: "Attrited Customer".to_string(),
            age: 30,
            one_hot_encoding: OneHotEncoding {
                education_level: "Graduate".to_string(),
                marital_status: "Single".to_string(),
                income_range: "$40K - $60K".to_string(),
                card_type: "Silver".to_string(),
            },
            mon_w_bank: 8,
            num_product_purchased: 3,
            mon_inactive: 3,
            num_contact: 12,
            card_credit_limit: 12000,
            evolving_bal: 800,
            transactions_amount: 3000,
            num_transctions: 15,
            avg_card_utilize: 0.3,
        }
    }
}    
