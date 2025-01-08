use crate::{bank::Bank, utils};

pub async fn save_bank(bank: &Bank) -> Result<(), tokio::io::Error>{
    let path = utils::expand_tilde(format!("~/.{}/bank.json", utils::APP_NAME).as_str());
    // create the directory if it does not exist
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    // serialize the bank to a json string
    let json = serde_json::to_string(bank).unwrap();
    // write the json string to the file
    tokio::fs::write(path, json).await?;
    Ok(())
}

pub async fn load_bank() -> Result<Bank, tokio::io::Error>{
    let path = utils::expand_tilde(format!("~/.{}/bank.json", utils::APP_NAME).as_str());
    // read the json string from the file
    let json = tokio::fs::read_to_string(path).await?;
    // deserialize the json string to a bank
    let bank = serde_json::from_str(json.as_str()).unwrap();
    Ok(bank)
}

