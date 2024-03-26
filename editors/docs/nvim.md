# NeoVim

## Prequisites
You have installed the `oxc_language_server` binary. Not needed if the extension can install/update it on its own

## Clients

### nvim-lsp

here is the configuration for nvim-lsp: 

```lua

-- oxc_language_server

local util = require('lspconfig.util')
local configs = require('lspconfig.configs')
local lspconfig = require("lspconfig")


configs.oxc_language_server = {
  default_config = {
    cmd = { 'oxc_language_server' },
    filetypes = {
      'javascript',
      'javascriptreact',
      'typescript',
      'typescriptreact',
    },
    root_dir = function(fname)
      return util.find_package_json_ancestor(fname)
          or util.find_node_modules_ancestor(fname)
          or util.find_git_ancestor(fname)
    end,
    single_file_support = true,
    -- settings is corresponding to https://github.com/oxc-project/oxc/blob/de2f83477444ea19e5e370419ab4678652c87d30/editors/vscode/package.json#L61C1-L95
    settings = {
      ['enable'] = true,
      ['run'] = 'onType'
    }
  },
}


lspconfig.oxc_language_server.setup {}
-- oxc_language_server end

```



