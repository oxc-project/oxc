import { jsx as _jsx, jsxs as _jsxs } from "react/jsx-runtime";
export function App() {
        return _jsxs(Suspense, {
                fallback: "Loading...",
                children: [_jsx(Init, {}), _jsxs(PanelGroup, {
                        direction: "horizontal",
                        className: "app-main",
                        children: [
                                _jsx(Panel, {
                                        defaultSize: 50,
                                        minSize: 33,
                                        maxSize: 66,
                                        children: _jsx(Input, {})
                                }),
                                _jsx(PanelResizeHandle, { className: "divider" }),
                                _jsx(Panel, { children: _jsx(Output, {}) })
                        ]
                })]
        });
}