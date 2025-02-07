// @target: es2018
// @experimentalDecorators: true
// @noEmitHelpers: true
// @noTypesAndSymbols: true
var BulkEditPreviewProvider_1;
let BulkEditPreviewProvider = class BulkEditPreviewProvider {
    static { BulkEditPreviewProvider_1 = this; }
    _modeService;
    static Schema = 'vscode-bulkeditpreview';
    static emptyPreview = { scheme: BulkEditPreviewProvider_1.Schema };
    constructor(_modeService) {
        this._modeService = _modeService;
    }
};
BulkEditPreviewProvider = BulkEditPreviewProvider_1 = babelHelpers.decorate([
    babelHelpers.decorateParam(0, IFoo)
], BulkEditPreviewProvider);
