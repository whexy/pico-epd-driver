-- A specific configuration for LazyVim to set configuration for rust-analyzer
---@type LazySpec
return {
	{
		"mrcjkb/rustaceanvim",
		opts = {
			server = {
				settings = {
					["rust-analyzer"] = {
						cargo = {
							target = "thumbv8m.main-none-eabihf",
							allTargets = false,
							loadOutDirsFromCheck = true, -- run build.rs
						},
						check = { command = "clippy", extraArgs = { "--target", "thumbv8m.main-none-eabihf" } },
					},
				},
			},
		},
	},
}
