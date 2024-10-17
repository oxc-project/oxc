export function App() {
	return (
		<Suspense fallback={"Loading..."}>
			<Init />
			<PanelGroup direction="horizontal" className="app-main">
				<Panel defaultSize={50} minSize={33} maxSize={66}>
					<Input />
				</Panel>
				<PanelResizeHandle className="divider" />
				<Panel>
					<Output />
				</Panel>
			</PanelGroup>
		</Suspense>
	);
}
