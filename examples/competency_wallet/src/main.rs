use anyhow::{anyhow, Result};

use bdk::{

    bitcoin::{bip32::ExtendedPrivKey, Network},

    blockchain::esplora::EsploraBlockchain,

    database::MemoryDatabase,

    keys::{

        bip39::{Language, Mnemonic, WordCount},

        DerivableKey, ExtendedKey, GeneratedKey, GeneratableKey,

    },

    wallet::{AddressIndex, SyncOptions, Wallet},

};



fn main() -> Result<()> {

    // ---------- connect to local Esplora (regtest electrs) ----------

    let base_url =

        std::env::var("ESPLORA_URL").unwrap_or_else(|_| "http://127.0.0.1:3002".into());

    let blockchain = EsploraBlockchain::new(&base_url, 20);



    // ---------- generate a 12-word mnemonic ----------

    let generated: GeneratedKey<_, bdk::descriptor::Segwitv0> =

        Mnemonic::generate((WordCount::Words12, Language::English))

            .map_err(|e| anyhow!("mnemonic generation failed: {e:?}"))?;

    println!("🔑  Mnemonic            : {}\n", *generated);



    // ---------- derive xprv ----------

    let xkey: ExtendedKey<_> = generated.into_extended_key()?;          // Result -> ?

    let xprv: ExtendedPrivKey = xkey

        .into_xprv(Network::Regtest)

        .ok_or_else(|| anyhow!("could not derive xprv from mnemonic"))?; // Option -> Result



    // ---------- build simple BIP-84 descriptors ----------

    let external_desc = format!("wpkh({}/84h/1h/0h/0/*)", xprv);

    let internal_desc = format!("wpkh({}/84h/1h/0h/1/*)", xprv);



    // ---------- create an in-memory wallet ----------

    let wallet = Wallet::new(

        &external_desc,

        Some(&internal_desc),

        Network::Regtest,

        MemoryDatabase::default(),

    )?;



    // ---------- sync & report ----------

    wallet.sync(&blockchain, SyncOptions::default())?;

    let addr = wallet.get_address(AddressIndex::New)?;

    println!("➡️  First receive addr   : {}", addr);

    println!("💰 Initial balance       : {} sats", wallet.get_balance()?);



    Ok(())

}