//solana config set --url https://api.mainnet-beta.solana.com
//solana program show TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
//solana program dump ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL /associated_token_program.so

#[cfg(test)]
mod tests {
    use anchor_lang::InstructionData;
    use anchor_lang::Space;

    use escrow::state::Escrow;
    use mollusk_svm::{program, result::Check, Mollusk};
    use solana_sdk::{
        account::{Account, WritableAccount},
        instruction::{AccountMeta, Instruction},
        native_token::LAMPORTS_PER_SOL,
        program_option::COption,
        program_pack::Pack,
        pubkey,
        pubkey::Pubkey,
    };
    use spl_associated_token_account::get_associated_token_address;
    use spl_token::state::AccountState;

    const ID: Pubkey = pubkey!("53E3gL8jErkT5PahCinHP6nw3P8ZtxBidvvLvsxpqs91");
    const ASSOCIATED_TOKEN_PROGRAM: Pubkey =
        pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
    const SEED: u64 = 1;
    const RECEIVE_AMOUNT: u64 = 10_000;
    const DEPOSIT_AMOUNT: u64 = 5_000;
    const MAKER: Pubkey = Pubkey::new_from_array([0x01; 32]);
    const TAKER: Pubkey = Pubkey::new_from_array([0x02; 32]);
    const MINT_X: Pubkey = Pubkey::new_from_array([0x03; 32]);
    const MINT_Y: Pubkey = Pubkey::new_from_array([0x04; 32]);
    const VAULT: Pubkey = Pubkey::new_from_array([0x09; 32]);

    #[test]
    fn test_make() {
        let mut mollusk = Mollusk::new(&ID, "../../target/deploy/escrow");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        mollusk.add_program(
            &spl_token::ID,
            "tests/elf/spl_token",
            &mollusk_svm::program::loader_keys::LOADER_V4,
        );

        mollusk.add_program(
            &ASSOCIATED_TOKEN_PROGRAM,
            "tests/elf/associated_token",
            &mollusk_svm::program::loader_keys::LOADER_V4,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        let (associated_program, associated_account) = (
            ASSOCIATED_TOKEN_PROGRAM,
            program::create_program_account_loader_v3(&ASSOCIATED_TOKEN_PROGRAM),
        );

        //get remanining pubkeys
        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &MAKER.to_bytes(), &SEED.to_le_bytes()],
            &ID,
        );
        let maker_ata_pubkey = get_associated_token_address(&MAKER, &MINT_X);
        let vault_pubkey = get_associated_token_address(&escrow, &MINT_X);

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
        let mut vault_account = Account::new(0, 0, &system_program);
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

        let data = escrow::instruction::Make {
            seed: SEED,
            receive: RECEIVE_AMOUNT,
            deposit: DEPOSIT_AMOUNT,
        }
        .data();

        //Make vec of Account Metas
        let ix_accs = vec![
            AccountMeta::new(MAKER, true),
            AccountMeta::new_readonly(MINT_X, false),
            AccountMeta::new_readonly(MINT_Y, false),
            AccountMeta::new(maker_ata_pubkey, false),
            AccountMeta::new(escrow, false),
            AccountMeta::new(vault_pubkey, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(associated_program, false),
            AccountMeta::new_readonly(system_program, false),
        ];

        //Make Instructiom
        let instruction = Instruction::new_with_bytes(ID, &data, ix_accs);

        //Make Transaction Accs Vec
        let tx_accs = vec![
            (MAKER, maker_account.clone()),
            (MINT_X, mint_x_account.clone()),
            (MINT_Y, mint_y_account.clone()),
            (maker_ata_pubkey, maker_ata_account.clone()),
            (escrow, escrow_account.clone()),
            (vault_pubkey, vault_account.clone()),
            (associated_program, associated_account.clone()),
            (token_program, token_account.clone()),
            (system_program, system_account.clone()),
        ];

        //Test
        mollusk.process_and_validate_instruction(&instruction, &tx_accs, &[Check::success()]);
    }

    #[test]
    fn test_take() {
        let mut mollusk = Mollusk::new(&ID, "../../target/deploy/escrow");

        let (system_program, system_account) =
            mollusk_svm::program::keyed_account_for_system_program();

        mollusk.add_program(
            &spl_token::ID,
            "tests/elf/spl_token",
            &mollusk_svm::program::loader_keys::LOADER_V4,
        );

        mollusk.add_program(
            &ASSOCIATED_TOKEN_PROGRAM,
            "tests/elf/associated_token",
            &mollusk_svm::program::loader_keys::LOADER_V4,
        );

        let (token_program, token_account) = (
            spl_token::ID,
            program::create_program_account_loader_v3(&spl_token::ID),
        );

        let (associated_program, associated_account) = (
            ASSOCIATED_TOKEN_PROGRAM,
            program::create_program_account_loader_v3(&ASSOCIATED_TOKEN_PROGRAM),
        );

        //get pubkeys
        let (escrow, escrow_bump) = solana_sdk::pubkey::Pubkey::find_program_address(
            &[(b"escrow"), &MAKER.to_bytes(), &SEED.to_le_bytes()],
            &ID,
        );
        let maker_ata_pubkey = get_associated_token_address(&MAKER, &MINT_Y);
        let taker_ata_x_pubkey = get_associated_token_address(&TAKER, &MINT_X);
        let taker_ata_y_pubkey = get_associated_token_address(&TAKER, &MINT_Y);

        let vault_pubkey = get_associated_token_address(&escrow, &MINT_X);

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
            &ID,
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
                owner: MAKER,
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
                mint: MINT_X,
                owner: TAKER,
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
                owner: TAKER,
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
            token_mint_a: MINT_X,
            token_mint_b: MINT_Y,
            receive_amount: RECEIVE_AMOUNT,
            bump: escrow_bump,
        };

        let mut escrow_writable_acc = escrow_account.data_as_mut_slice();
        anchor_lang::AccountSerialize::try_serialize(&escrow_data, &mut escrow_writable_acc)
            .expect("Failed to serialize state account data");

        let data = escrow::instruction::Exchange {}.data();

        //Make vec of Account Metas
        let ix_accs = vec![
            AccountMeta::new(TAKER, true),
            AccountMeta::new(MAKER, false),
            AccountMeta::new_readonly(MINT_X, false),
            AccountMeta::new_readonly(MINT_Y, false),
            AccountMeta::new(taker_ata_x_pubkey, false),
            AccountMeta::new(taker_ata_y_pubkey, false),
            AccountMeta::new(maker_ata_pubkey, false),
            AccountMeta::new(escrow, true),
            AccountMeta::new(vault_pubkey, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(associated_program, false),
            AccountMeta::new_readonly(system_program, false),
        ];

        //Make Instructiom
        let instruction = Instruction::new_with_bytes(ID, &data, ix_accs);

        //Make Transaction Accs Vec
        let tx_accs = vec![
            (TAKER, taker_account.clone()),
            (MAKER, maker_account.clone()),
            (MINT_X, mint_x_account.clone()),
            (MINT_Y, mint_y_account.clone()),
            (taker_ata_x_pubkey, taker_ata_x_account.clone()),
            (taker_ata_y_pubkey, taker_ata_y_account.clone()),
            (maker_ata_pubkey, maker_ata_y_account.clone()),
            (vault_pubkey, vault_account.clone()),
            (escrow, escrow_account.clone()),
            (associated_program, associated_account.clone()),
            (token_program, token_account.clone()),
            (system_program, system_account.clone()),
        ];

        //Test
        mollusk.process_and_validate_instruction(&instruction, &tx_accs, &[Check::success()]);
    }
}
