#[cfg(test)]
mod tests {

    #![no_std]
    extern crate alloc;

    use alloc::vec;
    use alloc::vec::Vec;
    use mollusk_svm::{program, result::Check, Mollusk};
    use pinocchio_log::log;
    use solana_sdk::{
        account::{Account, AccountSharedData, WritableAccount},
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        program_option::COption,
        program_pack::Pack,
        pubkey,
        pubkey::Pubkey,
        rent::Rent,
        sysvar::Sysvar,
    };
    use spl_token::state::AccountState;

    use crate::state::Escrow;

    const ID: Pubkey = pubkey!("A24MN2mj3aBpDLRhY6FonnbTuayv7oRqhva2R2hUuyqx");
    const SEED: u64 = 1;
    const RECEIVE_AMOUNT: u64 = 10_000;
    const DEPOSIT_AMOUNT: u64 = 5_000;
    const MAKER: Pubkey = Pubkey::new_from_array([0x01; 32]);
    const TAKER: Pubkey = Pubkey::new_from_array([0x02; 32]);
    const MINT_X: Pubkey = Pubkey::new_from_array([0x03; 32]);
    const MINT_Y: Pubkey = Pubkey::new_from_array([0x04; 32]);
    const MAKER_X_ATA: Pubkey = Pubkey::new_from_array([0x05; 32]);
    const TAKER_X_ATA: Pubkey = Pubkey::new_from_array([0x06; 32]);
    const MAKER_Y_ATA: Pubkey = Pubkey::new_from_array([0x07; 32]);
    const TAKER_Y_ATA: Pubkey = Pubkey::new_from_array([0x08; 32]);
    const VAULT: Pubkey = Pubkey::new_from_array([0x09; 32]);

    #[test]
    fn test_make() {
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        //get remanining pubkeys
        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &maker.to_bytes(), &SEED.to_le_bytes()],
            &ID,
        );

        //Make your Accounts DB
        let maker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
        let mut mint_x_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        let mut mint_y_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        let mut maker_ata_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        let mut vault_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        let escrow_account = Account::new(0, 0, &system_program);

        //Inject the data in to the accounts
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_x_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_y_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: MINT_X,
                owner: MAKER,
                amount: 20_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            maker_ata_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: MINT_X,
                owner: escrow,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        )
        .unwrap();

        let data = (vault::instruction::Deposit {
            seed: SEED,
            receive: RECEIVE_AMOUNT,
            deposit: DEPOSIT_AMOUNT,
        })
        .data();

        //Make vec of Account Metas
        let ix_accs = vec![
            AccountMeta::new(MAKER, true),
            AccountMeta::new_readonly(MINT_X, false),
            AccountMeta::new_readonly(MINT_Y, false),
            AccountMeta::new(MAKER_X_ATA, false),
            AccountMeta::new(VAULT, false),
            AccountMeta::new(escrow, true),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ];

        //Make Instructiom
        let instruction = Instruction::new_with_bytes(ID, &data, ix_accs);

        //Make Transaction Accs Vec
        let tx_accs = vec![
            (MAKER, maker_account),
            (MINT_X, mint_x_account),
            (MINT_Y, mint_y_account),
            (MAKER_X_ATA, maker_ata_account),
            (VAULT, vault_account),
            (escrow, escrow_account),
            (system_program, system_account),
            (token_program, token_account),
        ];

        //Test
        mollusk.process_and_validate_instruction(&instruction, &tx_accs, &[Check::success()]);
    }

    #[test]
    fn test_take() {
        let mut mollusk = Mollusk::new(&ID, "target/deploy/pinocchio_3");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        mollusk.add_program(
            &spl_token::ID,
            "src/tests/spl_token-3.5.0",
            &mollusk_svm::program::loader_keys::LOADER_V3,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        //get pubkeys
        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &maker.to_bytes(), &SEED.to_le_bytes()],
            &ID,
        );

        //Make your Accounts DB
        let taker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);
        let maker_account = Account::new(1 * LAMPORTS_PER_SOL, 0, &system_program);

        let mut mint_x_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        let mut mint_y_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Mint::LEN),
            spl_token::state::Mint::LEN,
            &token_program,
        );
        let mut taker_ata_x_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );

        let mut taker_ata_y_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );

        let mut maker_ata_y_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        let mut vault_account = Account::new(
            mollusk
                .sysvars
                .rent
                .minimum_balance(spl_token::state::Account::LEN),
            spl_token::state::Account::LEN,
            &token_program,
        );
        let mut escrow_account = Account::new(
            mollusk.sysvars.rent.minimum_balance(8 + Escrow::INIT_SPACE),
            8 + Escrow::INIT_SPACE,
            crate::ID,
        );

        //Inject the data in to the accounts
        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_x_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Mint {
                mint_authority: COption::None,
                supply: 100_000_000,
                decimals: 6,
                is_initialized: true,
                freeze_authority: COption::None,
            },
            mint_y_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: MINT_Y,
                owner: maker,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            maker_ata_y_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: MINT_Y,
                owner: maker,
                amount: 0,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            taker_ata_x_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: MINT_Y,
                owner: maker,
                amount: 10_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            taker_ata_y_account.data_as_mut_slice(),
        )
        .unwrap();

        solana_sdk::program_pack::Pack::pack(
            spl_token::state::Account {
                mint: MINT_X,
                owner: escrow,
                amount: 5_000,
                delegate: COption::None,
                state: AccountState::Initialized,
                is_native: COption::None,
                delegated_amount: 0,
                close_authority: COption::None,
            },
            vault_account.data_as_mut_slice(),
        )
        .unwrap();

        let escrow_data = Escrow {
            seed: SEED,
            maker: MAKER,
            token_mint_b: MINT_Y,
            receive_amount: RECEIVE_AMOUNT,
            token_mint_a: MINT_X,
            bump: escrow_bump,
        };

        anchor_lang::AccountSerialize::try_serialize(&escrow_data, &escrow_account)
            .expect("Failed to serialize state account data");

        let data = (vault::instruction::Exchange {}).data();

        //Make vec of Account Metas
        let ix_accs = vec![
            AccountMeta::new(TAKER, true),
            AccountMeta::new(MAKER, false),
            AccountMeta::new_readonly(MINT_X, false),
            AccountMeta::new_readonly(MINT_Y, false),
            AccountMeta::new(TAKER_X_ATA, false),
            AccountMeta::new(TAKER_Y_ATA, false),
            AccountMeta::new(MAKER_Y_ATA, false),
            AccountMeta::new(escrow, true),
            AccountMeta::new(VAULT, false),
            AccountMeta::new_readonly(system_program, false),
            AccountMeta::new_readonly(token_program, false),
        ];

        //Make Instructiom
        let instruction = Instruction::new_with_bytes(ID, &data, ix_accs);

        //Make Transaction Accs Vec
        let tx_accs = vec![
            (maker, maker_account),
            (mint_x, mint_x_account),
            (mint_y, mint_y_account),
            (maker_ata, maker_ata_account),
            (vault, vault_account),
            (escrow, escrow_account),
            (system_program, system_account),
            (token_program, token_account),
        ];

        //Test
        mollusk.process_and_validate_instruction(&instruction, &tx_accs, &[Check::success()]);
    }
}
