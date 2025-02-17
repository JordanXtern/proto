use proto_core::Tool;
use proto_pdk_api::*;

pub struct WasmTestWrapper {
    pub tool: Tool,
}

impl WasmTestWrapper {
    pub fn detect_version_files(&self) -> DetectVersionOutput {
        self.tool.plugin.call_func("detect_version_files").unwrap()
    }

    pub fn download_prebuilt(&self, mut input: DownloadPrebuiltInput) -> DownloadPrebuiltOutput {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_with("download_prebuilt", input)
            .unwrap()
    }

    pub fn load_versions(&self, input: LoadVersionsInput) -> LoadVersionsOutput {
        self.tool
            .plugin
            .call_func_with("load_versions", input)
            .unwrap()
    }

    pub fn locate_executables(&self, mut input: LocateExecutablesInput) -> LocateExecutablesOutput {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_with("locate_executables", input)
            .unwrap()
    }

    pub fn native_install(&self, mut input: NativeInstallInput) -> NativeInstallOutput {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_with("native_install", input)
            .unwrap()
    }

    pub fn native_uninstall(&self, mut input: NativeUninstallInput) -> NativeUninstallOutput {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_with("native_uninstall", input)
            .unwrap()
    }

    pub fn parse_version_file(&self, input: ParseVersionFileInput) -> ParseVersionFileOutput {
        self.tool
            .plugin
            .call_func_with("parse_version_file", input)
            .unwrap()
    }

    pub fn pre_install(&self, mut input: InstallHook) {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_without_output("pre_install", input)
            .unwrap();
    }

    pub fn pre_run(&self, mut input: RunHook) -> RunHookResult {
        input.context = self.prepare_context(input.context);

        self.tool.plugin.call_func_with("pre_run", input).unwrap()
    }

    pub fn post_install(&self, mut input: InstallHook) {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_without_output("post_install", input)
            .unwrap();
    }

    pub fn register_tool(&self, input: ToolMetadataInput) -> ToolMetadataOutput {
        self.tool
            .plugin
            .call_func_with("register_tool", input)
            .unwrap()
    }

    pub fn resolve_version(&self, input: ResolveVersionInput) -> ResolveVersionOutput {
        self.tool
            .plugin
            .call_func_with("resolve_version", input)
            .unwrap()
    }

    pub fn sync_manifest(&self, mut input: SyncManifestInput) -> SyncManifestOutput {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_with("sync_manifest", input)
            .unwrap()
    }

    pub fn sync_shell_profile(&self, mut input: SyncShellProfileInput) -> SyncShellProfileOutput {
        input.context = self.prepare_context(input.context);

        self.tool
            .plugin
            .call_func_with("sync_shell_profile", input)
            .unwrap()
    }

    pub fn unpack_archive(&self, mut input: UnpackArchiveInput) {
        input.input_file = self.tool.to_virtual_path(&input.input_file);
        input.output_dir = self.tool.to_virtual_path(&input.output_dir);

        let _: EmptyInput = self
            .tool
            .plugin
            .call_func_with("unpack_archive", input)
            .unwrap();
    }

    pub fn verify_checksum(&self, mut input: VerifyChecksumInput) -> VerifyChecksumOutput {
        input.checksum_file = self.tool.to_virtual_path(&input.checksum_file);
        input.download_file = self.tool.to_virtual_path(&input.download_file);

        self.tool
            .plugin
            .call_func_with("verify_checksum", input)
            .unwrap()
    }

    fn prepare_context(&self, context: ToolContext) -> ToolContext {
        let dir = if context.tool_dir.virtual_path().components().count() == 0 {
            self.tool.get_tool_dir()
        } else {
            context.tool_dir.virtual_path().to_path_buf()
        };

        ToolContext {
            tool_dir: self.tool.to_virtual_path(&dir),
            ..context
        }
    }
}
