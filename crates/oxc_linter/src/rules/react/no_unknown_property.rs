use oxc_diagnostics::{
    miette::{self, Diagnostic},
    thiserror::Error,
};
use oxc_macros::declare_oxc_lint;
use oxc_span::Span;

use crate::{context::LintContext, rule::Rule, AstNode};

#[derive(Debug, Error, Diagnostic)]
#[error("eslint-plugin-react(no-unknown-property):")]
#[diagnostic(severity(warning), help(""))]
struct NoUnknownPropertyDiagnostic(#[label] pub Span);

#[derive(Debug, Default, Clone)]
pub struct NoUnknownProperty;

declare_oxc_lint!(
    /// ### What it does
    /// Disallow usage of unknown DOM property.
    ///
    /// ### Why is this bad?
    /// You can use unknown property name that has no effect.
    ///
    /// ### Example
    /// ```javascript
    /// ```
    NoUnknownProperty,
    correctness
);

impl Rule for NoUnknownProperty {
    fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {}
}

#[test]
fn test() {
    use crate::tester::Tester;

    let pass = vec![
        (r#"<App class="bar" />;"#, None),
        (r#"<App for="bar" />;"#, None),
        (r#"<App someProp="bar" />;"#, None),
        (r#"<Foo.bar for="bar" />;"#, None),
        (r#"<App accept-charset="bar" />;"#, None),
        (r#"<App http-equiv="bar" />;"#, None),
        (r#"<App xlink:href="bar" />;"#, None),
        (r#"<App clip-path="bar" />;"#, None),
        (r#"<div className="bar"></div>;"#, None),
        (r#"<div onMouseDown={this._onMouseDown}></div>;"#, None),
        (r#"<a href="someLink" download="foo">Read more</a>"#, None),
        (r#"<area download="foo" />"#, None),
        (r#"<img src="cat_keyboard.jpeg" alt="A cat sleeping on a keyboard" align="top" />"#, None),
        (r#"<input type="password" required />"#, None),
        (r#"<input ref={this.input} type="radio" />"#, None),
        (r#"<input type="file" webkitdirectory="" />"#, None),
        (r#"<input type="file" webkitDirectory="" />"#, None),
        (r#"<div inert children="anything" />"#, None),
        (r#"<iframe scrolling="?" onLoad={a} onError={b} align="top" />"#, None),
        (r#"<input key="bar" type="radio" />"#, None),
        (r#"<button disabled>You cannot click me</button>;"#, None),
        (
            r#"<svg key="lock" viewBox="box" fill={10} d="d" stroke={1} strokeWidth={2} strokeLinecap={3} strokeLinejoin={4} transform="something" clipRule="else" x1={5} x2="6" y1="7" y2="8"></svg>"#,
            None,
        ),
        (r#"<g fill="\#7B82A0" fillRule="evenodd"></g>"#, None),
        (r#"<mask fill="\#7B82A0"></mask>"#, None),
        (r#"<symbol fill="\#7B82A0"></symbol>"#, None),
        (r#"<meta property="og:type" content="website" />"#, None),
        (
            r#"<input type="checkbox" checked={checked} disabled={disabled} id={id} onChange={onChange} />"#,
            None,
        ),
        (r#"<video playsInline />"#, None),
        (r#"<img onError={foo} onLoad={bar} />"#, None),
        (r#"<picture inert={false} onError={foo} onLoad={bar} />"#, None),
        (r#"<iframe onError={foo} onLoad={bar} />"#, None),
        (r#"<script onLoad={bar} onError={foo} />"#, None),
        (r#"<source onLoad={bar} onError={foo} />"#, None),
        (r#"<link onLoad={bar} onError={foo} />"#, None),
        (
            r#"<link rel="preload" as="image" href="someHref" imageSrcSet="someImageSrcSet" imageSizes="someImageSizes" />"#,
            None,
        ),
        (r#"<object onLoad={bar} />"#, None),
        (r#"<video allowFullScreen webkitAllowFullScreen mozAllowFullScreen />"#, None),
        (r#"<iframe allowFullScreen webkitAllowFullScreen mozAllowFullScreen />"#, None),
        (r#"<table border="1" />"#, None),
        (r#"<th abbr="abbr" />"#, None),
        (r#"<td abbr="abbr" />"#, None),
        (r#"<div allowTransparency="true" />"#, None),
        (r#"<div onPointerDown={this.onDown} onPointerUp={this.onUp} />"#, None),
        (r#"<input type="checkbox" defaultChecked={this.state.checkbox} />"#, None),
        (
            r#"<div onTouchStart={this.startAnimation} onTouchEnd={this.stopAnimation} onTouchCancel={this.cancel} onTouchMove={this.move} onMouseMoveCapture={this.capture} onTouchCancelCapture={this.log} />"#,
            None,
        ),
        (r#"<meta charset="utf-8" />;"#, None),
        (r#"<meta charSet="utf-8" />;"#, None),
        (r#"<div class="foo" is="my-elem"></div>;"#, None),
        (r#"<div {...this.props} class="foo" is="my-elem"></div>;"#, None),
        (r#"<atom-panel class="foo"></atom-panel>;"#, None),
        (r#"<div data-foo="bar"></div>;"#, None),
        (r#"<div data-foo-bar="baz"></div>;"#, None),
        (r#"<div data-parent="parent"></div>;"#, None),
        (r#"<div data-index-number="1234"></div>;"#, None),
        (r#"<div data-e2e-id="5678"></div>;"#, None),
        (r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#, None),
        (
            r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#,
            Some(serde_json::json!([{ "requireDataLowercase": false }])),
        ),
        (r#"<div class="bar"></div>;"#, Some(serde_json::json!([{ "ignore": ["class"] }]))),
        (r#"<div someProp="bar"></div>;"#, Some(serde_json::json!([{ "ignore": ["someProp"] }]))),
        (r#"<div css={{flex: 1}}></div>;"#, Some(serde_json::json!([{ "ignore": ["css"] }]))),
        (r#"<button aria-haspopup="true">Click me to open pop up</button>;"#, None),
        (r#"<button aria-label="Close" onClick={someThing.close} />;"#, None),
        (r#"<script crossOrigin noModule />"#, None),
        (r#"<audio crossOrigin />"#, None),
        (r#"<svg focusable><image crossOrigin /></svg>"#, None),
        (r#"<details onToggle={this.onToggle}>Some details</details>"#, None),
        (
            r#"<path fill="pink" d="M 10,30 A 20,20 0,0,1 50,30 A 20,20 0,0,1 90,30 Q 90,60 50,90 Q 10,60 10,30 z"></path>"#,
            None,
        ),
        (r#"<line fill="pink" x1="0" y1="80" x2="100" y2="20"></line>"#, None),
        (r#"<link as="audio">Audio content</link>"#, None),
        (
            r#"<video controlsList="nodownload" controls={this.controls} loop={true} muted={false} src={this.videoSrc} playsInline={true} onResize={this.onResize}></video>"#,
            None,
        ),
        (
            r#"<audio controlsList="nodownload" controls={this.controls} crossOrigin="anonymous" disableRemotePlayback loop muted preload="none" src="something" onAbort={this.abort} onDurationChange={this.durationChange} onEmptied={this.emptied} onEnded={this.end} onError={this.error} onResize={this.onResize}></audio>"#,
            None,
        ),
        (
            r#"<marker id={markerId} viewBox="0 0 2 2" refX="1" refY="1" markerWidth="1" markerHeight="1" orient="auto" />"#,
            None,
        ),
        (r#"<pattern id="pattern" viewBox="0,0,10,10" width="10%" height="10%" />"#, None),
        (r#"<symbol id="myDot" width="10" height="10" viewBox="0 0 2 2" />"#, None),
        (r#"<view id="one" viewBox="0 0 100 100" />"#, None),
        (r#"<hr align="top" />"#, None),
        (r#"<applet align="top" />"#, None),
        (r#"<marker fill="\#000" />"#, None),
        (
            r#"<dialog onClose={handler} open id="dialog" returnValue="something" onCancel={handler2} />"#,
            None,
        ),
        (
            r#"
			        <table align="top">
			          <caption align="top">Table Caption</caption>
			          <colgroup valign="top" align="top">
			            <col valign="top" align="top"/>
			          </colgroup>
			          <thead valign="top" align="top">
			            <tr valign="top" align="top">
			              <th valign="top" align="top">Header</th>
			              <td valign="top" align="top">Cell</td>
			            </tr>
			          </thead>
			          <tbody valign="top" align="top" />
			          <tfoot valign="top" align="top" />
			        </table>
			      "#,
            None,
        ),
        (r#"<fbt desc="foo" doNotExtract />;"#, None),
        (r#"<fbs desc="foo" doNotExtract />;"#, None),
        (r#"<math displaystyle="true" />;"#, None),
        (
            r#"
			        <div className="App" data-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash="customValue">
			          Hello, world!
			        </div>
			      "#,
            None,
        ),
    ];

    let fail = vec![
        (r#"<div allowTransparency="true" />"#, None),
        (r#"<div hasOwnProperty="should not be allowed property"></div>;"#, None),
        (r#"<div abc="should not be allowed property"></div>;"#, None),
        (r#"<div aria-fake="should not be allowed property"></div>;"#, None),
        (r#"<div someProp="bar"></div>;"#, None),
        (r#"<div class="bar"></div>;"#, None),
        (r#"<div for="bar"></div>;"#, None),
        (r#"<div accept-charset="bar"></div>;"#, None),
        (r#"<div http-equiv="bar"></div>;"#, None),
        (r#"<div accesskey="bar"></div>;"#, None),
        (r#"<div onclick="bar"></div>;"#, None),
        (r#"<div onmousedown="bar"></div>;"#, None),
        (r#"<div onMousedown="bar"></div>;"#, None),
        (r#"<use xlink:href="bar" />;"#, None),
        (r#"<rect clip-path="bar" />;"#, None),
        (r#"<script crossorigin nomodule />"#, None),
        (r#"<div crossorigin />"#, None),
        (r#"<div crossOrigin />"#, None),
        (r#"<div as="audio" />"#, None),
        (
            r#"<div onAbort={this.abort} onDurationChange={this.durationChange} onEmptied={this.emptied} onEnded={this.end} onResize={this.resize} onError={this.error} />"#,
            None,
        ),
        (r#"<div onLoad={this.load} />"#, None),
        (r#"<div fill="pink" />"#, None),
        (
            r#"<div controls={this.controls} loop={true} muted={false} src={this.videoSrc} playsInline={true} allowFullScreen></div>"#,
            None,
        ),
        (r#"<div download="foo" />"#, None),
        (r#"<div imageSrcSet="someImageSrcSet" />"#, None),
        (r#"<div imageSizes="someImageSizes" />"#, None),
        (r#"<div data-xml-anything="invalid" />"#, None),
        (
            r#"<div data-testID="bar" data-under_sCoRe="bar" />;"#,
            Some(serde_json::json!([{ "requireDataLowercase": true }])),
        ),
        (r#"<div abbr="abbr" />"#, None),
        (r#"<div webkitDirectory="" />"#, None),
        (r#"<div webkitdirectory="" />"#, None),
        (
            r#"
			        <div className="App" data-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash-crash:c="customValue">
			          Hello, world!
			        </div>
			      "#,
            None,
        ),
    ];

    Tester::new(NoUnknownProperty::NAME, pass, fail).test_and_snapshot();
}
