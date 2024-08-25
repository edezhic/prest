use anchor_lang::prelude::*;

declare_id!("S7SsyD8YqKtPzZS6pRButF658jCDnX5KvoU6kFQwKWH");

#[account]
#[derive(InitSpace)]
pub struct TodoList {
    #[max_len(5)]
    pub items: Vec<Todo>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub struct Todo {
    #[max_len(20)]
    pub task: String,
    pub done: bool,
}

#[program]
pub mod todo_solana {
    use super::*;

    #[derive(Accounts)]
    #[instruction(task: String)]
    pub struct AddTodo<'info> {
        #[account(
            init_if_needed,
            seeds = [owner.key().as_ref()],
            bump,
            payer = owner,
            space = 8 + TodoList::INIT_SPACE,
        )]
        pub list: Account<'info, TodoList>,
        #[account(mut)]
        pub owner: Signer<'info>,
        pub system_program: Program<'info, System>,
    }
    pub fn add_todo(ctx: Context<AddTodo>, task: String) -> Result<()> {
        ctx.accounts.list.items.push(Todo { task, done: false });
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(index: u32)]
    pub struct ToggleTodo<'info> {
        #[account(
            mut,
            seeds = [owner.key().as_ref()],
            bump,
        )]
        pub list: Account<'info, TodoList>,
        #[account(mut)]
        pub owner: Signer<'info>,
    }
    pub fn toggle_todo(ctx: Context<ToggleTodo>, index: u32) -> Result<()> {
        let index = index as usize;
        require!(ctx.accounts.list.items.get(index).is_some(), TodoError::NotFound);
        let todo = ctx.accounts.list.items.get_mut(index).unwrap();
        todo.done = !todo.done;
        Ok(())
    }

    #[derive(Accounts)]
    #[instruction(index: u32)]
    pub struct DeleteTodo<'info> {
        #[account(
            mut,
            seeds = [owner.key().as_ref()],
            bump,
        )]
        pub list: Account<'info, TodoList>,
        #[account(mut)]
        pub owner: Signer<'info>,
    }
    pub fn delete_todo(ctx: Context<DeleteTodo>, index: u32) -> Result<()> {
        let index = index as usize;
        require!(ctx.accounts.list.items.get(index).is_some(), TodoError::NotFound);
        ctx.accounts.list.items.remove(index);
        Ok(())
    }

    #[error_code]
    pub enum TodoError {
        NotFound
    }
}
