let object = {
	id: '_files.pickFolderAndOpen',
	handler: (
		accessor: ServicesAccessor,
		options: { forceNewWindow: boolean }
	) => accessor.get(IFileDialogService).pickFolderAndOpen(options)
}