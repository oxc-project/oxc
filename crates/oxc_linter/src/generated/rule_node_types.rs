// Auto-generated code for rule AST node type optimization
// DO NOT EDIT MANUALLY - run `just linter-codegen` to regenerate

use oxc_ast::AstType;

pub const TYPESCRIPTPROMISEFUNCTIONASYNC_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTPROMISEFUNCTIONASYNC_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOFLOATINGPROMISES_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOFLOATINGPROMISES_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNNECESSARYPARAMETERPROPERTYASSIGNMENT_NODE_TYPES: &[AstType] = &[AstType::MethodDefinition];
pub const TYPESCRIPTNOUNNECESSARYPARAMETERPROPERTYASSIGNMENT_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOUNNECESSARYBOOLEANLITERALCOMPARE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNNECESSARYBOOLEANLITERALCOMPARE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNONNULLABLETYPEASSERTIONSTYLE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNONNULLABLETYPEASSERTIONSTYLE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNONONNULLASSERTEDOPTIONALCHAIN_NODE_TYPES: &[AstType] = &[AstType::TSNonNullExpression];
pub const TYPESCRIPTNONONNULLASSERTEDOPTIONALCHAIN_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOUNSAFECALL_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFECALL_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTPREFERFUNCTIONTYPE_NODE_TYPES: &[AstType] = &[AstType::ExportDefaultDeclaration, AstType::TSInterfaceDeclaration, AstType::TSTypeAliasDeclaration, AstType::TSTypeAnnotation];
pub const TYPESCRIPTPREFERFUNCTIONTYPE_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTEXPLICITFUNCTIONRETURNTYPE_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::Function];
pub const TYPESCRIPTEXPLICITFUNCTIONRETURNTYPE_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTSWITCHEXHAUSTIVENESSCHECK_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTSWITCHEXHAUSTIVENESSCHECK_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNONONNULLASSERTION_NODE_TYPES: &[AstType] = &[AstType::TSNonNullExpression];
pub const TYPESCRIPTNONONNULLASSERTION_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTREQUIREARRAYSORTCOMPARE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTREQUIREARRAYSORTCOMPARE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNSAFEMEMBERACCESS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFEMEMBERACCESS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNODUPLICATETYPECONSTITUENTS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNODUPLICATETYPECONSTITUENTS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTPREFERENUMINITIALIZERS_NODE_TYPES: &[AstType] = &[AstType::TSEnumBody];
pub const TYPESCRIPTPREFERENUMINITIALIZERS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOARRAYDELETE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOARRAYDELETE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUSELESSEMPTYEXPORT_NODE_TYPES: &[AstType] = &[AstType::ExportNamedDeclaration];
pub const TYPESCRIPTNOUSELESSEMPTYEXPORT_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTRETURNAWAIT_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTRETURNAWAIT_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOMISUSEDSPREAD_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOMISUSEDSPREAD_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTRELATEDGETTERSETTERPAIRS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTRELATEDGETTERSETTERPAIRS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTARRAYTYPE_NODE_TYPES: &[AstType] = &[AstType::TSArrayType, AstType::TSAsExpression, AstType::TSConditionalType, AstType::TSIndexedAccessType, AstType::TSMappedType, AstType::TSTypeAliasDeclaration, AstType::TSTypeAnnotation, AstType::TSTypeParameterInstantiation, AstType::TSTypeReference];
pub const TYPESCRIPTARRAYTYPE_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTPREFERREDUCETYPEPARAMETER_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTPREFERREDUCETYPEPARAMETER_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTPREFERLITERALENUMMEMBER_NODE_TYPES: &[AstType] = &[AstType::TSEnumMember];
pub const TYPESCRIPTPREFERLITERALENUMMEMBER_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTUSEUNKNOWNINCATCHCALLBACKVARIABLE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTUSEUNKNOWNINCATCHCALLBACKVARIABLE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNODUPLICATEENUMVALUES_NODE_TYPES: &[AstType] = &[AstType::TSEnumBody];
pub const TYPESCRIPTNODUPLICATEENUMVALUES_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOMISUSEDPROMISES_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOMISUSEDPROMISES_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTPREFERFOROF_NODE_TYPES: &[AstType] = &[AstType::ForStatement];
pub const TYPESCRIPTPREFERFOROF_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTCONSISTENTINDEXEDOBJECTSTYLE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTCONSISTENTINDEXEDOBJECTSTYLE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNSAFEASSIGNMENT_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFEASSIGNMENT_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOREQUIREIMPORTS_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::TSImportEqualsDeclaration];
pub const TYPESCRIPTNOREQUIREIMPORTS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOREDUNDANTTYPECONSTITUENTS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOREDUNDANTTYPECONSTITUENTS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTPREFERNAMESPACEKEYWORD_NODE_TYPES: &[AstType] = &[AstType::TSModuleDeclaration];
pub const TYPESCRIPTPREFERNAMESPACEKEYWORD_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOCONFUSINGNONNULLASSERTION_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::BinaryExpression];
pub const TYPESCRIPTNOCONFUSINGNONNULLASSERTION_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOTHISALIAS_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::VariableDeclarator];
pub const TYPESCRIPTNOTHISALIAS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOUNSAFEENUMCOMPARISON_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFEENUMCOMPARISON_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOVARREQUIRES_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const TYPESCRIPTNOVARREQUIRES_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTADJACENTOVERLOADSIGNATURES_NODE_TYPES: &[AstType] = &[AstType::BlockStatement, AstType::Class, AstType::FunctionBody, AstType::Program, AstType::TSInterfaceDeclaration, AstType::TSModuleBlock, AstType::TSTypeLiteral];
pub const TYPESCRIPTADJACENTOVERLOADSIGNATURES_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTUNBOUNDMETHOD_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTUNBOUNDMETHOD_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOWRAPPEROBJECTTYPES_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOWRAPPEROBJECTTYPES_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOEXPLICITANY_NODE_TYPES: &[AstType] = &[AstType::TSAnyKeyword];
pub const TYPESCRIPTNOEXPLICITANY_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOIMPORTTYPESIDEEFFECTS_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const TYPESCRIPTNOIMPORTTYPESIDEEFFECTS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOUNSAFEFUNCTIONTYPE_NODE_TYPES: &[AstType] = &[AstType::TSClassImplements, AstType::TSInterfaceHeritage, AstType::TSTypeReference];
pub const TYPESCRIPTNOUNSAFEFUNCTIONTYPE_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOCONFUSINGVOIDEXPRESSION_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOCONFUSINGVOIDEXPRESSION_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNSAFEDECLARATIONMERGING_NODE_TYPES: &[AstType] = &[AstType::Class, AstType::TSInterfaceDeclaration];
pub const TYPESCRIPTNOUNSAFEDECLARATIONMERGING_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOMIXEDENUMS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOMIXEDENUMS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNNECESSARYTYPEASSERTION_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNNECESSARYTYPEASSERTION_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOBASETOSTRING_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOBASETOSTRING_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTCONSISTENTTYPEIMPORTS_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const TYPESCRIPTCONSISTENTTYPEIMPORTS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTPREFERTSEXPECTERROR_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTPREFERTSEXPECTERROR_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTRESTRICTPLUSOPERANDS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTRESTRICTPLUSOPERANDS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOIMPLIEDEVAL_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOIMPLIEDEVAL_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTONLYTHROWERROR_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTONLYTHROWERROR_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTAWAITTHENABLE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTAWAITTHENABLE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTPREFERRETURNTHISTYPE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTPREFERRETURNTHISTYPE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTPREFERASCONST_NODE_TYPES: &[AstType] = &[AstType::PropertyDefinition, AstType::TSAsExpression, AstType::VariableDeclarator];
pub const TYPESCRIPTPREFERASCONST_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTPREFERPROMISEREJECTERRORS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTPREFERPROMISEREJECTERRORS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOINFERRABLETYPES_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::Function, AstType::PropertyDefinition, AstType::VariableDeclarator];
pub const TYPESCRIPTNOINFERRABLETYPES_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTEXPLICITMODULEBOUNDARYTYPES_NODE_TYPES: &[AstType] = &[AstType::ExportDefaultDeclaration, AstType::ExportNamedDeclaration, AstType::TSExportAssignment];
pub const TYPESCRIPTEXPLICITMODULEBOUNDARYTYPES_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOMISUSEDNEW_NODE_TYPES: &[AstType] = &[AstType::Class, AstType::TSInterfaceDeclaration, AstType::TSMethodSignature];
pub const TYPESCRIPTNOMISUSEDNEW_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNODYNAMICDELETE_NODE_TYPES: &[AstType] = &[AstType::UnaryExpression];
pub const TYPESCRIPTNODYNAMICDELETE_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNONONNULLASSERTEDNULLISHCOALESCING_NODE_TYPES: &[AstType] = &[AstType::LogicalExpression];
pub const TYPESCRIPTNONONNULLASSERTEDNULLISHCOALESCING_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTBANTYPES_NODE_TYPES: &[AstType] = &[AstType::TSTypeLiteral, AstType::TSTypeReference];
pub const TYPESCRIPTBANTYPES_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOUNSAFERETURN_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFERETURN_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOMEANINGLESSVOIDOPERATOR_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOMEANINGLESSVOIDOPERATOR_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNNECESSARYTEMPLATEEXPRESSION_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNNECESSARYTEMPLATEEXPRESSION_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNSAFEUNARYMINUS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFEUNARYMINUS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOEMPTYOBJECTTYPE_NODE_TYPES: &[AstType] = &[AstType::TSInterfaceDeclaration, AstType::TSTypeLiteral];
pub const TYPESCRIPTNOEMPTYOBJECTTYPE_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTCONSISTENTTYPEDEFINITIONS_NODE_TYPES: &[AstType] = &[AstType::ExportDefaultDeclaration, AstType::TSInterfaceDeclaration, AstType::TSTypeAliasDeclaration];
pub const TYPESCRIPTCONSISTENTTYPEDEFINITIONS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOEXTRANEOUSCLASS_NODE_TYPES: &[AstType] = &[AstType::Class];
pub const TYPESCRIPTNOEXTRANEOUSCLASS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTCONSISTENTGENERICCONSTRUCTORS_NODE_TYPES: &[AstType] = &[AstType::AssignmentPattern, AstType::PropertyDefinition, AstType::VariableDeclarator];
pub const TYPESCRIPTCONSISTENTGENERICCONSTRUCTORS_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNONAMESPACE_NODE_TYPES: &[AstType] = &[AstType::TSModuleDeclaration];
pub const TYPESCRIPTNONAMESPACE_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTREQUIREAWAIT_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTREQUIREAWAIT_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTRESTRICTTEMPLATEEXPRESSIONS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTRESTRICTTEMPLATEEXPRESSIONS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTBANTSLINTCOMMENT_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTBANTSLINTCOMMENT_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOEXTRANONNULLASSERTION_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOEXTRANONNULLASSERTION_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNNECESSARYTYPECONSTRAINT_NODE_TYPES: &[AstType] = &[AstType::TSTypeParameterDeclaration];
pub const TYPESCRIPTNOUNNECESSARYTYPECONSTRAINT_ANY_NODE_TYPE: bool = false;

pub const TYPESCRIPTNOUNNECESSARYTYPEARGUMENTS_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNNECESSARYTYPEARGUMENTS_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNSAFEARGUMENT_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFEARGUMENT_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOFORINARRAY_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOFORINARRAY_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTBANTSCOMMENT_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTBANTSCOMMENT_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOUNSAFETYPEASSERTION_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTNOUNSAFETYPEASSERTION_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTTRIPLESLASHREFERENCE_NODE_TYPES: &[AstType] = &[];
pub const TYPESCRIPTTRIPLESLASHREFERENCE_ANY_NODE_TYPE: bool = true;

pub const TYPESCRIPTNOEMPTYINTERFACE_NODE_TYPES: &[AstType] = &[AstType::TSInterfaceDeclaration];
pub const TYPESCRIPTNOEMPTYINTERFACE_ANY_NODE_TYPE: bool = false;

pub const VITESTREQUIRELOCALTESTCONTEXTFORCONCURRENTSNAPSHOTS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const VITESTREQUIRELOCALTESTCONTEXTFORCONCURRENTSNAPSHOTS_ANY_NODE_TYPE: bool = false;

pub const VITESTPREFERTOBETRUTHY_NODE_TYPES: &[AstType] = &[];
pub const VITESTPREFERTOBETRUTHY_ANY_NODE_TYPE: bool = true;

pub const VITESTNOCONDITIONALTESTS_NODE_TYPES: &[AstType] = &[];
pub const VITESTNOCONDITIONALTESTS_ANY_NODE_TYPE: bool = true;

pub const VITESTNOIMPORTNODETEST_NODE_TYPES: &[AstType] = &[];
pub const VITESTNOIMPORTNODETEST_ANY_NODE_TYPE: bool = true;

pub const VITESTPREFERTOBEFALSY_NODE_TYPES: &[AstType] = &[];
pub const VITESTPREFERTOBEFALSY_ANY_NODE_TYPE: bool = true;

pub const VITESTPREFERTOBEOBJECT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const VITESTPREFERTOBEOBJECT_ANY_NODE_TYPE: bool = false;

pub const PROMISENORETURNINFINALLY_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const PROMISENORETURNINFINALLY_ANY_NODE_TYPE: bool = false;

pub const PROMISEPREFERCATCH_NODE_TYPES: &[AstType] = &[AstType::ExpressionStatement];
pub const PROMISEPREFERCATCH_ANY_NODE_TYPE: bool = false;

pub const PROMISENONESTING_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const PROMISENONESTING_ANY_NODE_TYPE: bool = false;

pub const PROMISENONEWSTATICS_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const PROMISENONEWSTATICS_ANY_NODE_TYPE: bool = false;

pub const PROMISESPECONLY_NODE_TYPES: &[AstType] = &[];
pub const PROMISESPECONLY_ANY_NODE_TYPE: bool = true;

pub const PROMISENOCALLBACKINPROMISE_NODE_TYPES: &[AstType] = &[];
pub const PROMISENOCALLBACKINPROMISE_ANY_NODE_TYPE: bool = true;

pub const PROMISEAVOIDNEW_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const PROMISEAVOIDNEW_ANY_NODE_TYPE: bool = false;

pub const PROMISENORETURNWRAP_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const PROMISENORETURNWRAP_ANY_NODE_TYPE: bool = false;

pub const PROMISENOPROMISEINCALLBACK_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const PROMISENOPROMISEINCALLBACK_ANY_NODE_TYPE: bool = false;

pub const PROMISEVALIDPARAMS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const PROMISEVALIDPARAMS_ANY_NODE_TYPE: bool = false;

pub const PROMISEPREFERAWAITTOCALLBACKS_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::CallExpression, AstType::Function];
pub const PROMISEPREFERAWAITTOCALLBACKS_ANY_NODE_TYPE: bool = false;

pub const PROMISEPARAMNAMES_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const PROMISEPARAMNAMES_ANY_NODE_TYPE: bool = false;

pub const PROMISECATCHORRETURN_NODE_TYPES: &[AstType] = &[AstType::ExpressionStatement];
pub const PROMISECATCHORRETURN_ANY_NODE_TYPE: bool = false;

pub const PROMISEPREFERAWAITTOTHEN_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const PROMISEPREFERAWAITTOTHEN_ANY_NODE_TYPE: bool = false;

pub const JESTPREFERHOOKSINORDER_NODE_TYPES: &[AstType] = &[];
pub const JESTPREFERHOOKSINORDER_ANY_NODE_TYPE: bool = true;

pub const JESTNOHOOKS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTNOHOOKS_ANY_NODE_TYPE: bool = false;

pub const JESTPREFEREQUALITYMATCHER_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFEREQUALITYMATCHER_ANY_NODE_TYPE: bool = false;

pub const JESTVALIDEXPECT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTVALIDEXPECT_ANY_NODE_TYPE: bool = false;

pub const JESTNOCONDITIONALINTEST_NODE_TYPES: &[AstType] = &[];
pub const JESTNOCONDITIONALINTEST_ANY_NODE_TYPE: bool = true;

pub const JESTEXPECTEXPECT_NODE_TYPES: &[AstType] = &[];
pub const JESTEXPECTEXPECT_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERCALLEDWITH_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFERCALLEDWITH_ANY_NODE_TYPE: bool = false;

pub const JESTNOLARGESNAPSHOTS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTNOLARGESNAPSHOTS_ANY_NODE_TYPE: bool = false;

pub const JESTNOTESTRETURNSTATEMENT_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::Function];
pub const JESTNOTESTRETURNSTATEMENT_ANY_NODE_TYPE: bool = false;

pub const JESTNOMOCKSIMPORT_NODE_TYPES: &[AstType] = &[];
pub const JESTNOMOCKSIMPORT_ANY_NODE_TYPE: bool = true;

pub const JESTNORESTRICTEDMATCHERS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTNORESTRICTEDMATCHERS_ANY_NODE_TYPE: bool = false;

pub const JESTNODUPLICATEHOOKS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTNODUPLICATEHOOKS_ANY_NODE_TYPE: bool = false;

pub const JESTNOCOMMENTEDOUTTESTS_NODE_TYPES: &[AstType] = &[];
pub const JESTNOCOMMENTEDOUTTESTS_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERSTRICTEQUAL_NODE_TYPES: &[AstType] = &[];
pub const JESTPREFERSTRICTEQUAL_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERCOMPARISONMATCHER_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFERCOMPARISONMATCHER_ANY_NODE_TYPE: bool = false;

pub const JESTNOINTERPOLATIONINSNAPSHOTS_NODE_TYPES: &[AstType] = &[];
pub const JESTNOINTERPOLATIONINSNAPSHOTS_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERTOCONTAIN_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFERTOCONTAIN_ANY_NODE_TYPE: bool = false;

pub const JESTPREFERTODO_NODE_TYPES: &[AstType] = &[];
pub const JESTPREFERTODO_ANY_NODE_TYPE: bool = true;

pub const NOSTANDALONEEXPECTMOD_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const NOSTANDALONEEXPECTMOD_ANY_NODE_TYPE: bool = false;

pub const JESTNODISABLEDTESTS_NODE_TYPES: &[AstType] = &[];
pub const JESTNODISABLEDTESTS_ANY_NODE_TYPE: bool = true;

pub const JESTNOJASMINEGLOBALS_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const JESTNOJASMINEGLOBALS_ANY_NODE_TYPE: bool = false;

pub const PREFERLOWERCASETITLEMOD_NODE_TYPES: &[AstType] = &[];
pub const PREFERLOWERCASETITLEMOD_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERTOHAVELENGTH_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFERTOHAVELENGTH_ANY_NODE_TYPE: bool = false;

pub const JESTVALIDTITLE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTVALIDTITLE_ANY_NODE_TYPE: bool = false;

pub const JESTPREFERHOOKSONTOP_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFERHOOKSONTOP_ANY_NODE_TYPE: bool = false;

pub const JESTPREFEREXPECTRESOLVES_NODE_TYPES: &[AstType] = &[];
pub const JESTPREFEREXPECTRESOLVES_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERJESTMOCKED_NODE_TYPES: &[AstType] = &[AstType::TSAsExpression];
pub const JESTPREFERJESTMOCKED_ANY_NODE_TYPE: bool = false;

pub const JESTVALIDDESCRIBECALLBACK_NODE_TYPES: &[AstType] = &[];
pub const JESTVALIDDESCRIBECALLBACK_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERMOCKPROMISESHORTHAND_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFERMOCKPROMISESHORTHAND_ANY_NODE_TYPE: bool = false;

pub const JESTNOCONFUSINGSETTIMEOUT_NODE_TYPES: &[AstType] = &[];
pub const JESTNOCONFUSINGSETTIMEOUT_ANY_NODE_TYPE: bool = true;

pub const JESTNOTESTPREFIXES_NODE_TYPES: &[AstType] = &[];
pub const JESTNOTESTPREFIXES_ANY_NODE_TYPE: bool = true;

pub const JESTNOCONDITIONALEXPECT_NODE_TYPES: &[AstType] = &[];
pub const JESTNOCONDITIONALEXPECT_ANY_NODE_TYPE: bool = true;

pub const JESTNOFOCUSEDTESTS_NODE_TYPES: &[AstType] = &[];
pub const JESTNOFOCUSEDTESTS_ANY_NODE_TYPE: bool = true;

pub const JESTPREFEREACH_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFEREACH_ANY_NODE_TYPE: bool = false;

pub const JESTNOIDENTICALTITLE_NODE_TYPES: &[AstType] = &[];
pub const JESTNOIDENTICALTITLE_ANY_NODE_TYPE: bool = true;

pub const JESTNORESTRICTEDJESTMETHODS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTNORESTRICTEDJESTMETHODS_ANY_NODE_TYPE: bool = false;

pub const JESTPREFERTOBE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTPREFERTOBE_ANY_NODE_TYPE: bool = false;

pub const JESTMAXEXPECTS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTMAXEXPECTS_ANY_NODE_TYPE: bool = false;

pub const JESTNODEPRECATEDFUNCTIONS_NODE_TYPES: &[AstType] = &[];
pub const JESTNODEPRECATEDFUNCTIONS_ANY_NODE_TYPE: bool = true;

pub const JESTREQUIREHOOK_NODE_TYPES: &[AstType] = &[AstType::Program];
pub const JESTREQUIREHOOK_ANY_NODE_TYPE: bool = false;

pub const JESTMAXNESTEDDESCRIBE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTMAXNESTEDDESCRIBE_ANY_NODE_TYPE: bool = false;

pub const JESTREQUIRETOTHROWMESSAGE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTREQUIRETOTHROWMESSAGE_ANY_NODE_TYPE: bool = false;

pub const JESTCONSISTENTTESTIT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTCONSISTENTTESTIT_ANY_NODE_TYPE: bool = false;

pub const JESTNODONECALLBACK_NODE_TYPES: &[AstType] = &[];
pub const JESTNODONECALLBACK_ANY_NODE_TYPE: bool = true;

pub const JESTREQUIRETOPLEVELDESCRIBE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTREQUIRETOPLEVELDESCRIBE_ANY_NODE_TYPE: bool = false;

pub const JESTNOALIASMETHODS_NODE_TYPES: &[AstType] = &[];
pub const JESTNOALIASMETHODS_ANY_NODE_TYPE: bool = true;

pub const JESTNOEXPORT_NODE_TYPES: &[AstType] = &[];
pub const JESTNOEXPORT_ANY_NODE_TYPE: bool = true;

pub const JESTPREFERSPYON_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const JESTPREFERSPYON_ANY_NODE_TYPE: bool = false;

pub const JESTNOUNTYPEDMOCKFACTORY_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const JESTNOUNTYPEDMOCKFACTORY_ANY_NODE_TYPE: bool = false;

pub const JSXA11YLANG_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YLANG_ANY_NODE_TYPE: bool = false;

pub const JSXA11YPREFERTAGOVERROLE_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YPREFERTAGOVERROLE_ANY_NODE_TYPE: bool = false;

pub const JSXA11YARIAACTIVEDESCENDANTHASTABINDEX_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YARIAACTIVEDESCENDANTHASTABINDEX_ANY_NODE_TYPE: bool = false;

pub const JSXA11YNOREDUNDANTROLES_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YNOREDUNDANTROLES_ANY_NODE_TYPE: bool = false;

pub const JSXA11YIMGREDUNDANTALT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YIMGREDUNDANTALT_ANY_NODE_TYPE: bool = false;

pub const JSXA11YARIAPROPS_NODE_TYPES: &[AstType] = &[AstType::JSXAttribute];
pub const JSXA11YARIAPROPS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YARIAUNSUPPORTEDELEMENTS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YARIAUNSUPPORTEDELEMENTS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YROLEHASREQUIREDARIAPROPS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YROLEHASREQUIREDARIAPROPS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YCLICKEVENTSHAVEKEYEVENTS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YCLICKEVENTSHAVEKEYEVENTS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YTABINDEXNOPOSITIVE_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YTABINDEXNOPOSITIVE_ANY_NODE_TYPE: bool = false;

pub const JSXA11YNOAUTOFOCUS_NODE_TYPES: &[AstType] = &[AstType::JSXElement];
pub const JSXA11YNOAUTOFOCUS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YLABELHASASSOCIATEDCONTROL_NODE_TYPES: &[AstType] = &[AstType::JSXElement];
pub const JSXA11YLABELHASASSOCIATEDCONTROL_ANY_NODE_TYPE: bool = false;

pub const JSXA11YMEDIAHASCAPTION_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YMEDIAHASCAPTION_ANY_NODE_TYPE: bool = false;

pub const JSXA11YAUTOCOMPLETEVALID_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YAUTOCOMPLETEVALID_ANY_NODE_TYPE: bool = false;

pub const JSXA11YSCOPE_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YSCOPE_ANY_NODE_TYPE: bool = false;

pub const JSXA11YARIAROLE_NODE_TYPES: &[AstType] = &[AstType::JSXElement];
pub const JSXA11YARIAROLE_ANY_NODE_TYPE: bool = false;

pub const JSXA11YNODISTRACTINGELEMENTS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YNODISTRACTINGELEMENTS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YHEADINGHASCONTENT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YHEADINGHASCONTENT_ANY_NODE_TYPE: bool = false;

pub const JSXA11YHTMLHASLANG_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YHTMLHASLANG_ANY_NODE_TYPE: bool = false;

pub const JSXA11YIFRAMEHASTITLE_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YIFRAMEHASTITLE_ANY_NODE_TYPE: bool = false;

pub const JSXA11YNONONINTERACTIVETABINDEX_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YNONONINTERACTIVETABINDEX_ANY_NODE_TYPE: bool = false;

pub const JSXA11YANCHORAMBIGUOUSTEXT_NODE_TYPES: &[AstType] = &[AstType::JSXElement];
pub const JSXA11YANCHORAMBIGUOUSTEXT_ANY_NODE_TYPE: bool = false;

pub const JSXA11YANCHORHASCONTENT_NODE_TYPES: &[AstType] = &[AstType::JSXElement];
pub const JSXA11YANCHORHASCONTENT_ANY_NODE_TYPE: bool = false;

pub const JSXA11YANCHORISVALID_NODE_TYPES: &[AstType] = &[AstType::JSXElement];
pub const JSXA11YANCHORISVALID_ANY_NODE_TYPE: bool = false;

pub const JSXA11YALTTEXT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YALTTEXT_ANY_NODE_TYPE: bool = false;

pub const JSXA11YNOACCESSKEY_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YNOACCESSKEY_ANY_NODE_TYPE: bool = false;

pub const JSXA11YMOUSEEVENTSHAVEKEYEVENTS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YMOUSEEVENTSHAVEKEYEVENTS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YROLESUPPORTSARIAPROPS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YROLESUPPORTSARIAPROPS_ANY_NODE_TYPE: bool = false;

pub const JSXA11YNOARIAHIDDENONFOCUSABLE_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const JSXA11YNOARIAHIDDENONFOCUSABLE_ANY_NODE_TYPE: bool = false;

pub const NODENONEWREQUIRE_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const NODENONEWREQUIRE_ANY_NODE_TYPE: bool = false;

pub const NODENOEXPORTSASSIGN_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const NODENOEXPORTSASSIGN_ANY_NODE_TYPE: bool = false;

pub const NEXTJSGOOGLEFONTDISPLAY_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSGOOGLEFONTDISPLAY_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOUNWANTEDPOLYFILLIO_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOUNWANTEDPOLYFILLIO_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOCSSTAGS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOCSSTAGS_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOHTMLLINKFORPAGES_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOHTMLLINKFORPAGES_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOSTYLEDJSXINDOCUMENT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOSTYLEDJSXINDOCUMENT_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNODUPLICATEHEAD_NODE_TYPES: &[AstType] = &[];
pub const NEXTJSNODUPLICATEHEAD_ANY_NODE_TYPE: bool = true;

pub const NEXTJSNOPAGECUSTOMFONT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOPAGECUSTOMFONT_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOHEADIMPORTINDOCUMENT_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const NEXTJSNOHEADIMPORTINDOCUMENT_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOBEFOREINTERACTIVESCRIPTOUTSIDEDOCUMENT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOBEFOREINTERACTIVESCRIPTOUTSIDEDOCUMENT_ANY_NODE_TYPE: bool = false;

pub const NEXTJSGOOGLEFONTPRECONNECT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSGOOGLEFONTPRECONNECT_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOSYNCSCRIPTS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOSYNCSCRIPTS_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOHEADELEMENT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOHEADELEMENT_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOIMGELEMENT_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNOIMGELEMENT_ANY_NODE_TYPE: bool = false;

pub const NEXTJSINLINESCRIPTID_NODE_TYPES: &[AstType] = &[AstType::ImportDefaultSpecifier];
pub const NEXTJSINLINESCRIPTID_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNEXTSCRIPTFORGA_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const NEXTJSNEXTSCRIPTFORGA_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOSCRIPTCOMPONENTINHEAD_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const NEXTJSNOSCRIPTCOMPONENTINHEAD_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOTITLEINDOCUMENTHEAD_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const NEXTJSNOTITLEINDOCUMENTHEAD_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNODOCUMENTIMPORTINPAGE_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const NEXTJSNODOCUMENTIMPORTINPAGE_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOASSIGNMODULEVARIABLE_NODE_TYPES: &[AstType] = &[AstType::VariableDeclaration];
pub const NEXTJSNOASSIGNMODULEVARIABLE_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOTYPOS_NODE_TYPES: &[AstType] = &[AstType::ExportNamedDeclaration];
pub const NEXTJSNOTYPOS_ANY_NODE_TYPE: bool = false;

pub const NEXTJSNOASYNCCLIENTCOMPONENT_NODE_TYPES: &[AstType] = &[];
pub const NEXTJSNOASYNCCLIENTCOMPONENT_ANY_NODE_TYPE: bool = true;

pub const IMPORTEXPORTSLAST_NODE_TYPES: &[AstType] = &[];
pub const IMPORTEXPORTSLAST_ANY_NODE_TYPE: bool = true;

pub const IMPORTEXPORT_NODE_TYPES: &[AstType] = &[];
pub const IMPORTEXPORT_ANY_NODE_TYPE: bool = true;

pub const IMPORTNONAMEDDEFAULT_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNONAMEDDEFAULT_ANY_NODE_TYPE: bool = true;

pub const IMPORTNOANONYMOUSDEFAULTEXPORT_NODE_TYPES: &[AstType] = &[AstType::ExportDefaultDeclaration];
pub const IMPORTNOANONYMOUSDEFAULTEXPORT_ANY_NODE_TYPE: bool = false;

pub const IMPORTNOWEBPACKLOADERSYNTAX_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::ImportDeclaration];
pub const IMPORTNOWEBPACKLOADERSYNTAX_ANY_NODE_TYPE: bool = false;

pub const IMPORTNAMED_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNAMED_ANY_NODE_TYPE: bool = true;

pub const IMPORTFIRST_NODE_TYPES: &[AstType] = &[];
pub const IMPORTFIRST_ANY_NODE_TYPE: bool = true;

pub const IMPORTNODUPLICATES_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNODUPLICATES_ANY_NODE_TYPE: bool = true;

pub const IMPORTNODEFAULTEXPORT_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNODEFAULTEXPORT_ANY_NODE_TYPE: bool = true;

pub const IMPORTCONSISTENTTYPESPECIFIERSTYLE_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const IMPORTCONSISTENTTYPESPECIFIERSTYLE_ANY_NODE_TYPE: bool = false;

pub const IMPORTGROUPEXPORTS_NODE_TYPES: &[AstType] = &[];
pub const IMPORTGROUPEXPORTS_ANY_NODE_TYPE: bool = true;

pub const IMPORTNONAMEDASDEFAULTMEMBER_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNONAMEDASDEFAULTMEMBER_ANY_NODE_TYPE: bool = true;

pub const IMPORTNAMESPACE_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNAMESPACE_ANY_NODE_TYPE: bool = true;

pub const IMPORTNOABSOLUTEPATH_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::ImportDeclaration];
pub const IMPORTNOABSOLUTEPATH_ANY_NODE_TYPE: bool = false;

pub const IMPORTEXTENSIONS_NODE_TYPES: &[AstType] = &[];
pub const IMPORTEXTENSIONS_ANY_NODE_TYPE: bool = true;

pub const IMPORTPREFERDEFAULTEXPORT_NODE_TYPES: &[AstType] = &[];
pub const IMPORTPREFERDEFAULTEXPORT_ANY_NODE_TYPE: bool = true;

pub const IMPORTNOMUTABLEEXPORTS_NODE_TYPES: &[AstType] = &[AstType::ExportDefaultDeclaration, AstType::ExportNamedDeclaration];
pub const IMPORTNOMUTABLEEXPORTS_ANY_NODE_TYPE: bool = false;

pub const IMPORTNOCYCLE_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNOCYCLE_ANY_NODE_TYPE: bool = true;

pub const IMPORTNOCOMMONJS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const IMPORTNOCOMMONJS_ANY_NODE_TYPE: bool = false;

pub const IMPORTNODYNAMICREQUIRE_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::ImportExpression];
pub const IMPORTNODYNAMICREQUIRE_ANY_NODE_TYPE: bool = false;

pub const IMPORTNONAMESPACE_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNONAMESPACE_ANY_NODE_TYPE: bool = true;

pub const IMPORTNOAMD_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const IMPORTNOAMD_ANY_NODE_TYPE: bool = false;

pub const IMPORTNOEMPTYNAMEDBLOCKS_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const IMPORTNOEMPTYNAMEDBLOCKS_ANY_NODE_TYPE: bool = false;

pub const IMPORTNOSELFIMPORT_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNOSELFIMPORT_ANY_NODE_TYPE: bool = true;

pub const IMPORTNONAMEDASDEFAULT_NODE_TYPES: &[AstType] = &[];
pub const IMPORTNONAMEDASDEFAULT_ANY_NODE_TYPE: bool = true;

pub const IMPORTUNAMBIGUOUS_NODE_TYPES: &[AstType] = &[];
pub const IMPORTUNAMBIGUOUS_ANY_NODE_TYPE: bool = true;

pub const IMPORTNOUNASSIGNEDIMPORT_NODE_TYPES: &[AstType] = &[AstType::ExpressionStatement, AstType::ImportDeclaration];
pub const IMPORTNOUNASSIGNEDIMPORT_ANY_NODE_TYPE: bool = false;

pub const IMPORTMAXDEPENDENCIES_NODE_TYPES: &[AstType] = &[];
pub const IMPORTMAXDEPENDENCIES_ANY_NODE_TYPE: bool = true;

pub const IMPORTDEFAULT_NODE_TYPES: &[AstType] = &[];
pub const IMPORTDEFAULT_ANY_NODE_TYPE: bool = true;

pub const JSDOCEMPTYTAGS_NODE_TYPES: &[AstType] = &[];
pub const JSDOCEMPTYTAGS_ANY_NODE_TYPE: bool = true;

pub const JSDOCIMPLEMENTSONCLASSES_NODE_TYPES: &[AstType] = &[];
pub const JSDOCIMPLEMENTSONCLASSES_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIREPROPERTYTYPE_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIREPROPERTYTYPE_ANY_NODE_TYPE: bool = true;

pub const JSDOCCHECKACCESS_NODE_TYPES: &[AstType] = &[];
pub const JSDOCCHECKACCESS_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIRERETURNS_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIRERETURNS_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIREPROPERTY_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIREPROPERTY_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIRERETURNSDESCRIPTION_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIRERETURNSDESCRIPTION_ANY_NODE_TYPE: bool = true;

pub const JSDOCCHECKPROPERTYNAMES_NODE_TYPES: &[AstType] = &[];
pub const JSDOCCHECKPROPERTYNAMES_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIRERETURNSTYPE_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIRERETURNSTYPE_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIREYIELDS_NODE_TYPES: &[AstType] = &[AstType::Function, AstType::YieldExpression];
pub const JSDOCREQUIREYIELDS_ANY_NODE_TYPE: bool = false;

pub const JSDOCREQUIREPARAMDESCRIPTION_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIREPARAMDESCRIPTION_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIREPARAMNAME_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIREPARAMNAME_ANY_NODE_TYPE: bool = true;

pub const JSDOCNODEFAULTS_NODE_TYPES: &[AstType] = &[];
pub const JSDOCNODEFAULTS_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIREPROPERTYDESCRIPTION_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIREPROPERTYDESCRIPTION_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIREPARAM_NODE_TYPES: &[AstType] = &[AstType::MethodDefinition];
pub const JSDOCREQUIREPARAM_ANY_NODE_TYPE: bool = false;

pub const JSDOCREQUIREPROPERTYNAME_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIREPROPERTYNAME_ANY_NODE_TYPE: bool = true;

pub const JSDOCCHECKTAGNAMES_NODE_TYPES: &[AstType] = &[];
pub const JSDOCCHECKTAGNAMES_ANY_NODE_TYPE: bool = true;

pub const JSDOCREQUIREPARAMTYPE_NODE_TYPES: &[AstType] = &[];
pub const JSDOCREQUIREPARAMTYPE_ANY_NODE_TYPE: bool = true;

pub const ESLINTNORESTRICTEDGLOBALS_NODE_TYPES: &[AstType] = &[AstType::IdentifierReference];
pub const ESLINTNORESTRICTEDGLOBALS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNONEWFUNC_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::NewExpression];
pub const ESLINTNONEWFUNC_ANY_NODE_TYPE: bool = false;

pub const ESLINTREQUIREYIELD_NODE_TYPES: &[AstType] = &[AstType::Function];
pub const ESLINTREQUIREYIELD_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUNDEF_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOUNDEF_ANY_NODE_TYPE: bool = true;

pub const ESLINTGETTERRETURN_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::Function];
pub const ESLINTGETTERRETURN_ANY_NODE_TYPE: bool = false;

pub const ESLINTRADIX_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTRADIX_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOGLOBALASSIGN_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOGLOBALASSIGN_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOUNUSEDEXPRESSIONS_NODE_TYPES: &[AstType] = &[AstType::ExpressionStatement];
pub const ESLINTNOUNUSEDEXPRESSIONS_ANY_NODE_TYPE: bool = false;

pub const ESLINTIDLENGTH_NODE_TYPES: &[AstType] = &[AstType::BindingIdentifier, AstType::IdentifierName, AstType::PrivateIdentifier];
pub const ESLINTIDLENGTH_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOSETTERRETURN_NODE_TYPES: &[AstType] = &[AstType::ReturnStatement];
pub const ESLINTNOSETTERRETURN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCONSTANTCONDITION_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression, AstType::DoWhileStatement, AstType::ForStatement, AstType::IfStatement, AstType::WhileStatement];
pub const ESLINTNOCONSTANTCONDITION_ANY_NODE_TYPE: bool = false;

pub const NOUNUSEDVARSUSAGE_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSUSAGE_ANY_NODE_TYPE: bool = true;

pub const NOUNUSEDVARSBINDINGPATTERN_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSBINDINGPATTERN_ANY_NODE_TYPE: bool = true;

pub const NOUNUSEDVARSDIAGNOSTIC_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSDIAGNOSTIC_ANY_NODE_TYPE: bool = true;

pub const NOUNUSEDVARSSYMBOL_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSSYMBOL_ANY_NODE_TYPE: bool = true;

pub const NOUNUSEDVARSOPTIONS_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSOPTIONS_ANY_NODE_TYPE: bool = true;

pub const NOUNUSEDVARSALLOWED_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSALLOWED_ANY_NODE_TYPE: bool = true;

pub const NOUNUSEDVARSIGNORED_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSIGNORED_ANY_NODE_TYPE: bool = true;

pub const NOUNUSEDVARSMOD_NODE_TYPES: &[AstType] = &[];
pub const NOUNUSEDVARSMOD_ANY_NODE_TYPE: bool = true;

pub const ESLINTNODUPECLASSMEMBERS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNODUPECLASSMEMBERS_ANY_NODE_TYPE: bool = true;

pub const ESLINTPREFEROBJECTSPREAD_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTPREFEROBJECTSPREAD_ANY_NODE_TYPE: bool = false;

pub const ESLINTYODA_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const ESLINTYODA_ANY_NODE_TYPE: bool = false;

pub const ESLINTFUNCNAMES_NODE_TYPES: &[AstType] = &[];
pub const ESLINTFUNCNAMES_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOLONELYIF_NODE_TYPES: &[AstType] = &[AstType::IfStatement];
pub const ESLINTNOLONELYIF_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOPLUSPLUS_NODE_TYPES: &[AstType] = &[AstType::UpdateExpression];
pub const ESLINTNOPLUSPLUS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOIRREGULARWHITESPACE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOIRREGULARWHITESPACE_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOARRAYCONSTRUCTOR_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOARRAYCONSTRUCTOR_ANY_NODE_TYPE: bool = true;

pub const ESLINTVALIDTYPEOF_NODE_TYPES: &[AstType] = &[];
pub const ESLINTVALIDTYPEOF_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOWITH_NODE_TYPES: &[AstType] = &[AstType::WithStatement];
pub const ESLINTNOWITH_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOSELFCOMPARE_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const ESLINTNOSELFCOMPARE_ANY_NODE_TYPE: bool = false;

pub const ESLINTMAXLINESPERFUNCTION_NODE_TYPES: &[AstType] = &[];
pub const ESLINTMAXLINESPERFUNCTION_ANY_NODE_TYPE: bool = true;

pub const ESLINTDEFAULTPARAMLAST_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::Function];
pub const ESLINTDEFAULTPARAMLAST_ANY_NODE_TYPE: bool = false;

pub const ESLINTUSEISNAN_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression, AstType::CallExpression, AstType::SwitchCase, AstType::SwitchStatement];
pub const ESLINTUSEISNAN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOOBJECTCONSTRUCTOR_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOOBJECTCONSTRUCTOR_ANY_NODE_TYPE: bool = true;

pub const ESLINTSORTVARS_NODE_TYPES: &[AstType] = &[AstType::VariableDeclaration];
pub const ESLINTSORTVARS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUNUSEDPRIVATECLASSMEMBERS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOUNUSEDPRIVATECLASSMEMBERS_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOTHROWLITERAL_NODE_TYPES: &[AstType] = &[AstType::ThrowStatement];
pub const ESLINTNOTHROWLITERAL_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOMULTISTR_NODE_TYPES: &[AstType] = &[AstType::StringLiteral];
pub const ESLINTNOMULTISTR_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEMPTYPATTERN_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOEMPTYPATTERN_ANY_NODE_TYPE: bool = true;

pub const ESLINTNODELETEVAR_NODE_TYPES: &[AstType] = &[AstType::UnaryExpression];
pub const ESLINTNODELETEVAR_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOLOSSOFPRECISION_NODE_TYPES: &[AstType] = &[AstType::NumericLiteral];
pub const ESLINTNOLOSSOFPRECISION_ANY_NODE_TYPE: bool = false;

pub const ESLINTSORTIMPORTS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTSORTIMPORTS_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOLABELS_NODE_TYPES: &[AstType] = &[AstType::LabeledStatement];
pub const ESLINTNOLABELS_ANY_NODE_TYPE: bool = false;

pub const ESLINTPREFERSPREAD_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTPREFERSPREAD_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOSPARSEARRAYS_NODE_TYPES: &[AstType] = &[AstType::ArrayExpression];
pub const ESLINTNOSPARSEARRAYS_ANY_NODE_TYPE: bool = false;

pub const ESLINTMAXPARAMS_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::Function];
pub const ESLINTMAXPARAMS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEMPTYSTATICBLOCK_NODE_TYPES: &[AstType] = &[AstType::StaticBlock];
pub const ESLINTNOEMPTYSTATICBLOCK_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOVAR_NODE_TYPES: &[AstType] = &[AstType::VariableDeclaration];
pub const ESLINTNOVAR_ANY_NODE_TYPE: bool = false;

pub const ESLINTCURLY_NODE_TYPES: &[AstType] = &[AstType::DoWhileStatement, AstType::ForInStatement, AstType::ForOfStatement, AstType::ForStatement, AstType::IfStatement, AstType::WhileStatement];
pub const ESLINTCURLY_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOSCRIPTURL_NODE_TYPES: &[AstType] = &[AstType::StringLiteral, AstType::TemplateLiteral];
pub const ESLINTNOSCRIPTURL_ANY_NODE_TYPE: bool = false;

pub const ESLINTSYMBOLDESCRIPTION_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTSYMBOLDESCRIPTION_ANY_NODE_TYPE: bool = false;

pub const ESLINTNONESTEDTERNARY_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression];
pub const ESLINTNONESTEDTERNARY_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUSELESSCONCAT_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const ESLINTNOUSELESSCONCAT_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOITERATOR_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOITERATOR_ANY_NODE_TYPE: bool = true;

pub const ESLINTNODEBUGGER_NODE_TYPES: &[AstType] = &[AstType::DebuggerStatement];
pub const ESLINTNODEBUGGER_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOIMPORTASSIGN_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOIMPORTASSIGN_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOUNSAFEFINALLY_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOUNSAFEFINALLY_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOTEMPLATECURLYINSTRING_NODE_TYPES: &[AstType] = &[AstType::StringLiteral];
pub const ESLINTNOTEMPLATECURLYINSTRING_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOFUNCASSIGN_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOFUNCASSIGN_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOEMPTYCHARACTERCLASS_NODE_TYPES: &[AstType] = &[AstType::RegExpLiteral];
pub const ESLINTNOEMPTYCHARACTERCLASS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEXTRABIND_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTNOEXTRABIND_ANY_NODE_TYPE: bool = false;

pub const ESLINTSORTKEYS_NODE_TYPES: &[AstType] = &[AstType::ObjectExpression];
pub const ESLINTSORTKEYS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOREGEXSPACES_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::NewExpression, AstType::RegExpLiteral];
pub const ESLINTNOREGEXSPACES_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCLASSASSIGN_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOCLASSASSIGN_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOEVAL_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::Program, AstType::ThisExpression];
pub const ESLINTNOEVAL_ANY_NODE_TYPE: bool = false;

pub const ESLINTEQEQEQ_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const ESLINTEQEQEQ_ANY_NODE_TYPE: bool = false;

pub const ESLINTNONEGATEDCONDITION_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNONEGATEDCONDITION_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOUNUSEDLABELS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOUNUSEDLABELS_ANY_NODE_TYPE: bool = true;

pub const ESLINTPREFERNUMERICLITERALS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTPREFERNUMERICLITERALS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUSELESSCONSTRUCTOR_NODE_TYPES: &[AstType] = &[AstType::MethodDefinition];
pub const ESLINTNOUSELESSCONSTRUCTOR_ANY_NODE_TYPE: bool = false;

pub const ESLINTBLOCKSCOPEDVAR_NODE_TYPES: &[AstType] = &[AstType::VariableDeclaration];
pub const ESLINTBLOCKSCOPEDVAR_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOBITWISE_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::BinaryExpression, AstType::UnaryExpression];
pub const ESLINTNOBITWISE_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEQNULL_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const ESLINTNOEQNULL_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCASEDECLARATIONS_NODE_TYPES: &[AstType] = &[AstType::SwitchCase];
pub const ESLINTNOCASEDECLARATIONS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNONONOCTALDECIMALESCAPE_NODE_TYPES: &[AstType] = &[AstType::StringLiteral];
pub const ESLINTNONONOCTALDECIMALESCAPE_ANY_NODE_TYPE: bool = false;

pub const ESLINTNEWCAP_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::NewExpression];
pub const ESLINTNEWCAP_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOAWAITINLOOP_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOAWAITINLOOP_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOUNASSIGNEDVARS_NODE_TYPES: &[AstType] = &[AstType::VariableDeclarator];
pub const ESLINTNOUNASSIGNEDVARS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEMPTY_NODE_TYPES: &[AstType] = &[AstType::BlockStatement, AstType::SwitchStatement];
pub const ESLINTNOEMPTY_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCONTROLREGEX_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOCONTROLREGEX_ANY_NODE_TYPE: bool = true;

pub const ESLINTDEFAULTCASE_NODE_TYPES: &[AstType] = &[AstType::SwitchStatement];
pub const ESLINTDEFAULTCASE_ANY_NODE_TYPE: bool = false;

pub const ESLINTNODUPLICATECASE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNODUPLICATECASE_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOELSERETURN_NODE_TYPES: &[AstType] = &[AstType::IfStatement];
pub const ESLINTNOELSERETURN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOREDECLARE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOREDECLARE_ANY_NODE_TYPE: bool = true;

pub const ESLINTOPERATORASSIGNMENT_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const ESLINTOPERATORASSIGNMENT_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOMAGICNUMBERS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOMAGICNUMBERS_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOUSELESSRENAME_NODE_TYPES: &[AstType] = &[AstType::ExportNamedDeclaration, AstType::ImportSpecifier, AstType::ObjectAssignmentTarget, AstType::ObjectPattern];
pub const ESLINTNOUSELESSRENAME_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEXTENDNATIVE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOEXTENDNATIVE_ANY_NODE_TYPE: bool = true;

pub const ESLINTFORDIRECTION_NODE_TYPES: &[AstType] = &[AstType::ForStatement];
pub const ESLINTFORDIRECTION_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOINNERDECLARATIONS_NODE_TYPES: &[AstType] = &[AstType::Function, AstType::VariableDeclaration];
pub const ESLINTNOINNERDECLARATIONS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOMULTIASSIGN_NODE_TYPES: &[AstType] = &[AstType::VariableDeclarator];
pub const ESLINTNOMULTIASSIGN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNORESTRICTEDIMPORTS_NODE_TYPES: &[AstType] = &[AstType::TSImportEqualsDeclaration];
pub const ESLINTNORESTRICTEDIMPORTS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCONSTASSIGN_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOCONSTASSIGN_ANY_NODE_TYPE: bool = true;

pub const ESLINTGROUPEDACCESSORPAIRS_NODE_TYPES: &[AstType] = &[AstType::ClassBody, AstType::ObjectExpression];
pub const ESLINTGROUPEDACCESSORPAIRS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNODUPLICATEIMPORTS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNODUPLICATEIMPORTS_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOVOID_NODE_TYPES: &[AstType] = &[AstType::UnaryExpression];
pub const ESLINTNOVOID_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUNNEEDEDTERNARY_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression];
pub const ESLINTNOUNNEEDEDTERNARY_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCOMPARENEGZERO_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const ESLINTNOCOMPARENEGZERO_ANY_NODE_TYPE: bool = false;

pub const ESLINTNONEW_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const ESLINTNONEW_ANY_NODE_TYPE: bool = false;

pub const ESLINTPREFERPROMISEREJECTERRORS_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::NewExpression];
pub const ESLINTPREFERPROMISEREJECTERRORS_ANY_NODE_TYPE: bool = false;

pub const ESLINTINITDECLARATIONS_NODE_TYPES: &[AstType] = &[AstType::VariableDeclaration];
pub const ESLINTINITDECLARATIONS_ANY_NODE_TYPE: bool = false;

pub const ESLINTPREFERRESTPARAMS_NODE_TYPES: &[AstType] = &[AstType::IdentifierReference];
pub const ESLINTPREFERRESTPARAMS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEXTRALABEL_NODE_TYPES: &[AstType] = &[AstType::BreakStatement];
pub const ESLINTNOEXTRALABEL_ANY_NODE_TYPE: bool = false;

pub const ESLINTPREFEREXPONENTIATIONOPERATOR_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTPREFEREXPONENTIATIONOPERATOR_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCONDASSIGN_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::ConditionalExpression, AstType::DoWhileStatement, AstType::ForStatement, AstType::IfStatement, AstType::WhileStatement];
pub const ESLINTNOCONDASSIGN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUSELESSCALL_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTNOUSELESSCALL_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOALERT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTNOALERT_ANY_NODE_TYPE: bool = false;

pub const ESLINTNODUPEKEYS_NODE_TYPES: &[AstType] = &[AstType::ObjectExpression];
pub const ESLINTNODUPEKEYS_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUNEXPECTEDMULTILINE_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression, AstType::CallExpression, AstType::ComputedMemberExpression, AstType::TaggedTemplateExpression];
pub const ESLINTNOUNEXPECTEDMULTILINE_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUSELESSBACKREFERENCE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOUSELESSBACKREFERENCE_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOCONSTRUCTORRETURN_NODE_TYPES: &[AstType] = &[AstType::ReturnStatement];
pub const ESLINTNOCONSTRUCTORRETURN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCALLER_NODE_TYPES: &[AstType] = &[AstType::StaticMemberExpression];
pub const ESLINTNOCALLER_ANY_NODE_TYPE: bool = false;

pub const ESLINTMAXDEPTH_NODE_TYPES: &[AstType] = &[];
pub const ESLINTMAXDEPTH_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOPROTOTYPEBUILTINS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTNOPROTOTYPEBUILTINS_ANY_NODE_TYPE: bool = false;

pub const ESLINTUNICODEBOM_NODE_TYPES: &[AstType] = &[];
pub const ESLINTUNICODEBOM_ANY_NODE_TYPE: bool = true;

pub const ESLINTDEFAULTCASELAST_NODE_TYPES: &[AstType] = &[AstType::SwitchStatement];
pub const ESLINTDEFAULTCASELAST_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUNREACHABLE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOUNREACHABLE_ANY_NODE_TYPE: bool = true;

pub const ESLINTPREFERDESTRUCTURING_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::VariableDeclarator];
pub const ESLINTPREFERDESTRUCTURING_ANY_NODE_TYPE: bool = false;

pub const ESLINTNONEWNATIVENONCONSTRUCTOR_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const ESLINTNONEWNATIVENONCONSTRUCTOR_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUNSAFENEGATION_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const ESLINTNOUNSAFENEGATION_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUSELESSESCAPE_NODE_TYPES: &[AstType] = &[AstType::RegExpLiteral, AstType::StringLiteral, AstType::TemplateLiteral];
pub const ESLINTNOUSELESSESCAPE_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOLONEBLOCKS_NODE_TYPES: &[AstType] = &[AstType::BlockStatement];
pub const ESLINTNOLONEBLOCKS_ANY_NODE_TYPE: bool = false;

pub const ARRAYCALLBACKRETURNRETURNCHECKER_NODE_TYPES: &[AstType] = &[];
pub const ARRAYCALLBACKRETURNRETURNCHECKER_ANY_NODE_TYPE: bool = true;

pub const ARRAYCALLBACKRETURNMOD_NODE_TYPES: &[AstType] = &[];
pub const ARRAYCALLBACKRETURNMOD_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOCONSTANTBINARYEXPRESSION_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression, AstType::LogicalExpression];
pub const ESLINTNOCONSTANTBINARYEXPRESSION_ANY_NODE_TYPE: bool = false;

pub const ESLINTVARSONTOP_NODE_TYPES: &[AstType] = &[AstType::VariableDeclaration];
pub const ESLINTVARSONTOP_ANY_NODE_TYPE: bool = false;

pub const ESLINTMAXLINES_NODE_TYPES: &[AstType] = &[];
pub const ESLINTMAXLINES_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOUNDEFINED_NODE_TYPES: &[AstType] = &[AstType::BindingIdentifier, AstType::IdentifierReference];
pub const ESLINTNOUNDEFINED_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOTERNARY_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression];
pub const ESLINTNOTERNARY_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOOBJCALLS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOOBJCALLS_ANY_NODE_TYPE: bool = true;

pub const ESLINTNORETURNASSIGN_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const ESLINTNORETURNASSIGN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOSHADOWRESTRICTEDNAMES_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOSHADOWRESTRICTEDNAMES_ANY_NODE_TYPE: bool = true;

pub const ESLINTMAXNESTEDCALLBACKS_NODE_TYPES: &[AstType] = &[];
pub const ESLINTMAXNESTEDCALLBACKS_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOASYNCPROMISEEXECUTOR_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const ESLINTNOASYNCPROMISEEXECUTOR_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOCONSOLE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOCONSOLE_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOFALLTHROUGH_NODE_TYPES: &[AstType] = &[AstType::SwitchStatement];
pub const ESLINTNOFALLTHROUGH_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEMPTYFUNCTION_NODE_TYPES: &[AstType] = &[AstType::FunctionBody];
pub const ESLINTNOEMPTYFUNCTION_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOUSELESSCATCH_NODE_TYPES: &[AstType] = &[AstType::TryStatement];
pub const ESLINTNOUSELESSCATCH_ANY_NODE_TYPE: bool = false;

pub const ESLINTNONEWWRAPPERS_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const ESLINTNONEWWRAPPERS_ANY_NODE_TYPE: bool = false;

pub const ESLINTREQUIREAWAIT_NODE_TYPES: &[AstType] = &[AstType::FunctionBody];
pub const ESLINTREQUIREAWAIT_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOTHISBEFORESUPER_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOTHISBEFORESUPER_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOEXTRABOOLEANCAST_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::UnaryExpression];
pub const ESLINTNOEXTRABOOLEANCAST_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOINVALIDREGEXP_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOINVALIDREGEXP_ANY_NODE_TYPE: bool = true;

pub const ESLINTFUNCSTYLE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTFUNCSTYLE_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOLABELVAR_NODE_TYPES: &[AstType] = &[AstType::LabeledStatement];
pub const ESLINTNOLABELVAR_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOPROTO_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOPROTO_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOUNSAFEOPTIONALCHAINING_NODE_TYPES: &[AstType] = &[AstType::Argument, AstType::ArrayExpression, AstType::AssignmentExpression, AstType::AssignmentPattern, AstType::AssignmentTargetWithDefault, AstType::BinaryExpression, AstType::CallExpression, AstType::Class, AstType::ComputedMemberExpression, AstType::ForOfStatement, AstType::NewExpression, AstType::PrivateFieldExpression, AstType::StaticMemberExpression, AstType::TaggedTemplateExpression, AstType::UnaryExpression, AstType::VariableDeclarator, AstType::WithStatement];
pub const ESLINTNOUNSAFEOPTIONALCHAINING_ANY_NODE_TYPE: bool = false;

pub const ESLINTGUARDFORIN_NODE_TYPES: &[AstType] = &[AstType::ForInStatement];
pub const ESLINTGUARDFORIN_ANY_NODE_TYPE: bool = false;

pub const ESLINTPREFEROBJECTHASOWN_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const ESLINTPREFEROBJECTHASOWN_ANY_NODE_TYPE: bool = false;

pub const ESLINTNODIVREGEX_NODE_TYPES: &[AstType] = &[AstType::RegExpLiteral];
pub const ESLINTNODIVREGEX_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOEXASSIGN_NODE_TYPES: &[AstType] = &[];
pub const ESLINTNOEXASSIGN_ANY_NODE_TYPE: bool = true;

pub const ESLINTMAXCLASSESPERFILE_NODE_TYPES: &[AstType] = &[];
pub const ESLINTMAXCLASSESPERFILE_ANY_NODE_TYPE: bool = true;

pub const ESLINTNOCONTINUE_NODE_TYPES: &[AstType] = &[AstType::ContinueStatement];
pub const ESLINTNOCONTINUE_ANY_NODE_TYPE: bool = false;

pub const ESLINTNODUPEELSEIF_NODE_TYPES: &[AstType] = &[AstType::IfStatement];
pub const ESLINTNODUPEELSEIF_ANY_NODE_TYPE: bool = false;

pub const ESLINTARROWBODYSTYLE_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression];
pub const ESLINTARROWBODYSTYLE_ANY_NODE_TYPE: bool = false;

pub const ESLINTNOSELFASSIGN_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const ESLINTNOSELFASSIGN_ANY_NODE_TYPE: bool = false;

pub const REACTPERFJSXNONEWOBJECTASPROP_NODE_TYPES: &[AstType] = &[];
pub const REACTPERFJSXNONEWOBJECTASPROP_ANY_NODE_TYPE: bool = true;

pub const REACTPERFJSXNONEWARRAYASPROP_NODE_TYPES: &[AstType] = &[];
pub const REACTPERFJSXNONEWARRAYASPROP_ANY_NODE_TYPE: bool = true;

pub const REACTPERFJSXNONEWFUNCTIONASPROP_NODE_TYPES: &[AstType] = &[];
pub const REACTPERFJSXNONEWFUNCTIONASPROP_ANY_NODE_TYPE: bool = true;

pub const REACTPERFJSXNOJSXASPROP_NODE_TYPES: &[AstType] = &[];
pub const REACTPERFJSXNOJSXASPROP_ANY_NODE_TYPE: bool = true;

pub const OXCMISREFACTOREDASSIGNOP_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const OXCMISREFACTOREDASSIGNOP_ANY_NODE_TYPE: bool = false;

pub const OXCONLYUSEDINRECURSION_NODE_TYPES: &[AstType] = &[];
pub const OXCONLYUSEDINRECURSION_ANY_NODE_TYPE: bool = true;

pub const OXCNOACCUMULATINGSPREAD_NODE_TYPES: &[AstType] = &[AstType::SpreadElement];
pub const OXCNOACCUMULATINGSPREAD_ANY_NODE_TYPE: bool = false;

pub const OXCNOOPTIONALCHAINING_NODE_TYPES: &[AstType] = &[AstType::ChainExpression];
pub const OXCNOOPTIONALCHAINING_ANY_NODE_TYPE: bool = false;

pub const OXCNOCONSTENUM_NODE_TYPES: &[AstType] = &[AstType::TSEnumDeclaration];
pub const OXCNOCONSTENUM_ANY_NODE_TYPE: bool = false;

pub const OXCBADREPLACEALLARG_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const OXCBADREPLACEALLARG_ANY_NODE_TYPE: bool = false;

pub const OXCBADARRAYMETHODONARGUMENTS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const OXCBADARRAYMETHODONARGUMENTS_ANY_NODE_TYPE: bool = false;

pub const OXCBADCHARATCOMPARISON_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const OXCBADCHARATCOMPARISON_ANY_NODE_TYPE: bool = false;

pub const OXCNOBARRELFILE_NODE_TYPES: &[AstType] = &[];
pub const OXCNOBARRELFILE_ANY_NODE_TYPE: bool = true;

pub const OXCNORESTSPREADPROPERTIES_NODE_TYPES: &[AstType] = &[AstType::BindingRestElement, AstType::ObjectAssignmentTarget, AstType::SpreadElement];
pub const OXCNORESTSPREADPROPERTIES_ANY_NODE_TYPE: bool = false;

pub const OXCDOUBLECOMPARISONS_NODE_TYPES: &[AstType] = &[AstType::LogicalExpression];
pub const OXCDOUBLECOMPARISONS_ANY_NODE_TYPE: bool = false;

pub const OXCBADBITWISEOPERATOR_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::BinaryExpression];
pub const OXCBADBITWISEOPERATOR_ANY_NODE_TYPE: bool = false;

pub const OXCMISSINGTHROW_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const OXCMISSINGTHROW_ANY_NODE_TYPE: bool = false;

pub const OXCAPPROXCONSTANT_NODE_TYPES: &[AstType] = &[AstType::NumericLiteral];
pub const OXCAPPROXCONSTANT_ANY_NODE_TYPE: bool = false;

pub const OXCBADMINMAXFUNC_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const OXCBADMINMAXFUNC_ANY_NODE_TYPE: bool = false;

pub const OXCBADCOMPARISONSEQUENCE_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const OXCBADCOMPARISONSEQUENCE_ANY_NODE_TYPE: bool = false;

pub const OXCUNINVOKEDARRAYCALLBACK_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const OXCUNINVOKEDARRAYCALLBACK_ANY_NODE_TYPE: bool = false;

pub const OXCERASINGOP_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const OXCERASINGOP_ANY_NODE_TYPE: bool = false;

pub const OXCNUMBERARGOUTOFRANGE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const OXCNUMBERARGOUTOFRANGE_ANY_NODE_TYPE: bool = false;

pub const OXCCONSTCOMPARISONS_NODE_TYPES: &[AstType] = &[];
pub const OXCCONSTCOMPARISONS_ANY_NODE_TYPE: bool = true;

pub const OXCNOASYNCENDPOINTHANDLERS_NODE_TYPES: &[AstType] = &[];
pub const OXCNOASYNCENDPOINTHANDLERS_ANY_NODE_TYPE: bool = true;

pub const OXCBADOBJECTLITERALCOMPARISON_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const OXCBADOBJECTLITERALCOMPARISON_ANY_NODE_TYPE: bool = false;

pub const OXCNOASYNCAWAIT_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::Function];
pub const OXCNOASYNCAWAIT_ANY_NODE_TYPE: bool = false;

pub const OXCNOMAPSPREAD_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const OXCNOMAPSPREAD_ANY_NODE_TYPE: bool = false;

pub const REACTJSXNOSCRIPTURL_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const REACTJSXNOSCRIPTURL_ANY_NODE_TYPE: bool = false;

pub const REACTCHECKEDREQUIRESONCHANGEORREADONLY_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXOpeningElement];
pub const REACTCHECKEDREQUIRESONCHANGEORREADONLY_ANY_NODE_TYPE: bool = false;

pub const REACTRULESOFHOOKS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const REACTRULESOFHOOKS_ANY_NODE_TYPE: bool = false;

pub const REACTJSXNOUNDEF_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const REACTJSXNOUNDEF_ANY_NODE_TYPE: bool = false;

pub const REACTJSXFRAGMENTS_NODE_TYPES: &[AstType] = &[AstType::JSXElement, AstType::JSXFragment];
pub const REACTJSXFRAGMENTS_ANY_NODE_TYPE: bool = false;

pub const REACTJSXCURLYBRACEPRESENCE_NODE_TYPES: &[AstType] = &[AstType::JSXElement, AstType::JSXFragment];
pub const REACTJSXCURLYBRACEPRESENCE_ANY_NODE_TYPE: bool = false;

pub const REACTREQUIRERENDERRETURN_NODE_TYPES: &[AstType] = &[];
pub const REACTREQUIRERENDERRETURN_ANY_NODE_TYPE: bool = true;

pub const REACTNOSTRINGREFS_NODE_TYPES: &[AstType] = &[AstType::JSXAttribute];
pub const REACTNOSTRINGREFS_ANY_NODE_TYPE: bool = false;

pub const REACTJSXNODUPLICATEPROPS_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const REACTJSXNODUPLICATEPROPS_ANY_NODE_TYPE: bool = false;

pub const REACTSTYLEPROPOBJECT_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXElement];
pub const REACTSTYLEPROPOBJECT_ANY_NODE_TYPE: bool = false;

pub const REACTNOCHILDRENPROP_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXAttribute];
pub const REACTNOCHILDRENPROP_ANY_NODE_TYPE: bool = false;

pub const REACTEXHAUSTIVEDEPS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const REACTEXHAUSTIVEDEPS_ANY_NODE_TYPE: bool = false;

pub const REACTNODIRECTMUTATIONSTATE_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::UpdateExpression];
pub const REACTNODIRECTMUTATIONSTATE_ANY_NODE_TYPE: bool = false;

pub const REACTJSXPROPSNOSPREADMULTI_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const REACTJSXPROPSNOSPREADMULTI_ANY_NODE_TYPE: bool = false;

pub const REACTJSXBOOLEANVALUE_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const REACTJSXBOOLEANVALUE_ANY_NODE_TYPE: bool = false;

pub const REACTIFRAMEMISSINGSANDBOX_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXOpeningElement];
pub const REACTIFRAMEMISSINGSANDBOX_ANY_NODE_TYPE: bool = false;

pub const REACTNODANGERWITHCHILDREN_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXElement];
pub const REACTNODANGERWITHCHILDREN_ANY_NODE_TYPE: bool = false;

pub const REACTJSXNOTARGETBLANK_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const REACTJSXNOTARGETBLANK_ANY_NODE_TYPE: bool = false;

pub const REACTNOISMOUNTED_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const REACTNOISMOUNTED_ANY_NODE_TYPE: bool = false;

pub const REACTPREFERES6CLASS_NODE_TYPES: &[AstType] = &[];
pub const REACTPREFERES6CLASS_ANY_NODE_TYPE: bool = true;

pub const REACTFORBIDELEMENTS_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXOpeningElement];
pub const REACTFORBIDELEMENTS_ANY_NODE_TYPE: bool = false;

pub const REACTJSXFILENAMEEXTENSION_NODE_TYPES: &[AstType] = &[];
pub const REACTJSXFILENAMEEXTENSION_ANY_NODE_TYPE: bool = true;

pub const REACTVOIDDOMELEMENTSNOCHILDREN_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXElement];
pub const REACTVOIDDOMELEMENTSNOCHILDREN_ANY_NODE_TYPE: bool = false;

pub const REACTNOSETSTATE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const REACTNOSETSTATE_ANY_NODE_TYPE: bool = false;

pub const REACTBUTTONHASTYPE_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXOpeningElement];
pub const REACTBUTTONHASTYPE_ANY_NODE_TYPE: bool = false;

pub const REACTNOUNESCAPEDENTITIES_NODE_TYPES: &[AstType] = &[AstType::JSXText];
pub const REACTNOUNESCAPEDENTITIES_ANY_NODE_TYPE: bool = false;

pub const REACTNOFINDDOMNODE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const REACTNOFINDDOMNODE_ANY_NODE_TYPE: bool = false;

pub const REACTREACTINJSXSCOPE_NODE_TYPES: &[AstType] = &[];
pub const REACTREACTINJSXSCOPE_ANY_NODE_TYPE: bool = true;

pub const REACTSELFCLOSINGCOMP_NODE_TYPES: &[AstType] = &[AstType::JSXElement];
pub const REACTSELFCLOSINGCOMP_ANY_NODE_TYPE: bool = false;

pub const REACTJSXNOUSELESSFRAGMENT_NODE_TYPES: &[AstType] = &[AstType::JSXElement, AstType::JSXFragment];
pub const REACTJSXNOUSELESSFRAGMENT_ANY_NODE_TYPE: bool = false;

pub const REACTFORWARDREFUSESREF_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const REACTFORWARDREFUSESREF_ANY_NODE_TYPE: bool = false;

pub const REACTJSXKEY_NODE_TYPES: &[AstType] = &[AstType::JSXElement, AstType::JSXFragment];
pub const REACTJSXKEY_ANY_NODE_TYPE: bool = false;

pub const REACTNONAMESPACE_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXOpeningElement];
pub const REACTNONAMESPACE_ANY_NODE_TYPE: bool = false;

pub const REACTJSXNOCOMMENTTEXTNODES_NODE_TYPES: &[AstType] = &[AstType::JSXText];
pub const REACTJSXNOCOMMENTTEXTNODES_ANY_NODE_TYPE: bool = false;

pub const REACTNORENDERRETURNVALUE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const REACTNORENDERRETURNVALUE_ANY_NODE_TYPE: bool = false;

pub const REACTNODANGER_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXElement];
pub const REACTNODANGER_ANY_NODE_TYPE: bool = false;

pub const REACTNOARRAYINDEXKEY_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::JSXElement];
pub const REACTNOARRAYINDEXKEY_ANY_NODE_TYPE: bool = false;

pub const REACTNOUNKNOWNPROPERTY_NODE_TYPES: &[AstType] = &[AstType::JSXOpeningElement];
pub const REACTNOUNKNOWNPROPERTY_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOMAGICARRAYFLATDEPTH_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOMAGICARRAYFLATDEPTH_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFEROBJECTFROMENTRIES_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFEROBJECTFROMENTRIES_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFEREVENTTARGET_NODE_TYPES: &[AstType] = &[AstType::IdentifierReference];
pub const UNICORNPREFEREVENTTARGET_ANY_NODE_TYPE: bool = false;

pub const UNICORNNUMBERLITERALCASE_NODE_TYPES: &[AstType] = &[];
pub const UNICORNNUMBERLITERALCASE_ANY_NODE_TYPE: bool = true;

pub const UNICORNNOTHISASSIGNMENT_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::VariableDeclarator];
pub const UNICORNNOTHISASSIGNMENT_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERSETHAS_NODE_TYPES: &[AstType] = &[AstType::VariableDeclarator];
pub const UNICORNPREFERSETHAS_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERARRAYFIND_NODE_TYPES: &[AstType] = &[AstType::ComputedMemberExpression];
pub const UNICORNPREFERARRAYFIND_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOLONELYIF_NODE_TYPES: &[AstType] = &[AstType::IfStatement];
pub const UNICORNNOLONELYIF_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOTYPEOFUNDEFINED_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const UNICORNNOTYPEOFUNDEFINED_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERSETSIZE_NODE_TYPES: &[AstType] = &[AstType::StaticMemberExpression];
pub const UNICORNPREFERSETSIZE_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERMATHMINMAX_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression];
pub const UNICORNPREFERMATHMINMAX_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERSTRINGTRIMSTARTEND_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERSTRINGTRIMSTARTEND_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERNODEPROTOCOL_NODE_TYPES: &[AstType] = &[];
pub const UNICORNPREFERNODEPROTOCOL_ANY_NODE_TYPE: bool = true;

pub const UNICORNNOUSELESSSWITCHCASE_NODE_TYPES: &[AstType] = &[AstType::SwitchStatement];
pub const UNICORNNOUSELESSSWITCHCASE_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOANONYMOUSDEFAULTEXPORT_NODE_TYPES: &[AstType] = &[];
pub const UNICORNNOANONYMOUSDEFAULTEXPORT_ANY_NODE_TYPE: bool = true;

pub const UNICORNREQUIREARRAYJOINSEPARATOR_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNREQUIREARRAYJOINSEPARATOR_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERSTRINGREPLACEALL_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERSTRINGREPLACEALL_ANY_NODE_TYPE: bool = false;

pub const UNICORNFILENAMECASE_NODE_TYPES: &[AstType] = &[];
pub const UNICORNFILENAMECASE_ANY_NODE_TYPE: bool = true;

pub const UNICORNNOACCESSORRECURSION_NODE_TYPES: &[AstType] = &[AstType::ThisExpression];
pub const UNICORNNOACCESSORRECURSION_ANY_NODE_TYPE: bool = false;

pub const UNICORNNONEWARRAY_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const UNICORNNONEWARRAY_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERMODERNMATHAPIS_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression, AstType::CallExpression];
pub const UNICORNPREFERMODERNMATHAPIS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOUSELESSFALLBACKINSPREAD_NODE_TYPES: &[AstType] = &[AstType::LogicalExpression];
pub const UNICORNNOUSELESSFALLBACKINSPREAD_ANY_NODE_TYPE: bool = false;

pub const UNICORNCONSISTENTEMPTYARRAYSPREAD_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression];
pub const UNICORNCONSISTENTEMPTYARRAYSPREAD_ANY_NODE_TYPE: bool = false;

pub const UNICORNCONSISTENTEXISTENCEINDEXCHECK_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const UNICORNCONSISTENTEXISTENCEINDEXCHECK_ANY_NODE_TYPE: bool = false;

pub const UNICORNEMPTYBRACESPACES_NODE_TYPES: &[AstType] = &[];
pub const UNICORNEMPTYBRACESPACES_ANY_NODE_TYPE: bool = true;

pub const UNICORNNOARRAYMETHODTHISARGUMENT_NODE_TYPES: &[AstType] = &[];
pub const UNICORNNOARRAYMETHODTHISARGUMENT_ANY_NODE_TYPE: bool = true;

pub const UNICORNPREFERSTRINGRAW_NODE_TYPES: &[AstType] = &[AstType::StringLiteral];
pub const UNICORNPREFERSTRINGRAW_ANY_NODE_TYPE: bool = false;

pub const UNICORNCONSISTENTDATECLONE_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const UNICORNCONSISTENTDATECLONE_ANY_NODE_TYPE: bool = false;

pub const UNICORNCONSISTENTASSERT_NODE_TYPES: &[AstType] = &[AstType::ImportDeclaration];
pub const UNICORNCONSISTENTASSERT_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOINVALIDFETCHOPTIONS_NODE_TYPES: &[AstType] = &[];
pub const UNICORNNOINVALIDFETCHOPTIONS_ANY_NODE_TYPE: bool = true;

pub const UNICORNPREFERSPREAD_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERSPREAD_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERDOMNODEAPPEND_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERDOMNODEAPPEND_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERARRAYINDEXOF_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERARRAYINDEXOF_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFEROPTIONALCATCHBINDING_NODE_TYPES: &[AstType] = &[AstType::CatchParameter];
pub const UNICORNPREFEROPTIONALCATCHBINDING_ANY_NODE_TYPE: bool = false;

pub const UNICORNSWITCHCASEBRACES_NODE_TYPES: &[AstType] = &[AstType::SwitchStatement];
pub const UNICORNSWITCHCASEBRACES_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERARRAYSOME_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression, AstType::CallExpression];
pub const UNICORNPREFERARRAYSOME_ANY_NODE_TYPE: bool = false;

pub const UNICORNNONEWBUFFER_NODE_TYPES: &[AstType] = &[AstType::NewExpression];
pub const UNICORNNONEWBUFFER_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERNATIVECOERCIONFUNCTIONS_NODE_TYPES: &[AstType] = &[AstType::ArrowFunctionExpression, AstType::Function];
pub const UNICORNPREFERNATIVECOERCIONFUNCTIONS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNODOCUMENTCOOKIE_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const UNICORNNODOCUMENTCOOKIE_ANY_NODE_TYPE: bool = false;

pub const UNICORNNONESTEDTERNARY_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression];
pub const UNICORNNONESTEDTERNARY_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOZEROFRACTIONS_NODE_TYPES: &[AstType] = &[AstType::NumericLiteral];
pub const UNICORNNOZEROFRACTIONS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOAWAITINPROMISEMETHODS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOAWAITINPROMISEMETHODS_ANY_NODE_TYPE: bool = false;

pub const UNICORNTEXTENCODINGIDENTIFIERCASE_NODE_TYPES: &[AstType] = &[];
pub const UNICORNTEXTENCODINGIDENTIFIERCASE_ANY_NODE_TYPE: bool = true;

pub const UNICORNTHROWNEWERROR_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNTHROWNEWERROR_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOUSELESSLENGTHCHECK_NODE_TYPES: &[AstType] = &[AstType::LogicalExpression];
pub const UNICORNNOUSELESSLENGTHCHECK_ANY_NODE_TYPE: bool = false;

pub const UNICORNCONSISTENTFUNCTIONSCOPING_NODE_TYPES: &[AstType] = &[];
pub const UNICORNCONSISTENTFUNCTIONSCOPING_ANY_NODE_TYPE: bool = true;

pub const UNICORNPREFERINCLUDES_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const UNICORNPREFERINCLUDES_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERPROTOTYPEMETHODS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERPROTOTYPEMETHODS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNONEGATIONINEQUALITYCHECK_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const UNICORNNONEGATIONINEQUALITYCHECK_ANY_NODE_TYPE: bool = false;

pub const UNICORNEXPLICITLENGTHCHECK_NODE_TYPES: &[AstType] = &[AstType::StaticMemberExpression];
pub const UNICORNEXPLICITLENGTHCHECK_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOARRAYREDUCE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOARRAYREDUCE_ANY_NODE_TYPE: bool = false;

pub const UNICORNREQUIREPOSTMESSAGETARGETORIGIN_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNREQUIREPOSTMESSAGETARGETORIGIN_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERARRAYFLAT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERARRAYFLAT_ANY_NODE_TYPE: bool = false;

pub const UNICORNCATCHERRORNAME_NODE_TYPES: &[AstType] = &[AstType::CatchParameter];
pub const UNICORNCATCHERRORNAME_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOEMPTYFILE_NODE_TYPES: &[AstType] = &[];
pub const UNICORNNOEMPTYFILE_ANY_NODE_TYPE: bool = true;

pub const UNICORNNOUNNECESSARYAWAIT_NODE_TYPES: &[AstType] = &[AstType::AwaitExpression];
pub const UNICORNNOUNNECESSARYAWAIT_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOUNNECESSARYSLICEEND_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOUNNECESSARYSLICEEND_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERARRAYFLATMAP_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERARRAYFLATMAP_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOINVALIDREMOVEEVENTLISTENER_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOINVALIDREMOVEEVENTLISTENER_ANY_NODE_TYPE: bool = false;

pub const UNICORNNUMERICSEPARATORSSTYLE_NODE_TYPES: &[AstType] = &[AstType::BigIntLiteral, AstType::NumericLiteral];
pub const UNICORNNUMERICSEPARATORSSTYLE_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOINSTANCEOFBUILTINS_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const UNICORNNOINSTANCEOFBUILTINS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOTHENABLE_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::CallExpression, AstType::ExportNamedDeclaration, AstType::MethodDefinition, AstType::ObjectExpression, AstType::PropertyDefinition];
pub const UNICORNNOTHENABLE_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOPROCESSEXIT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOPROCESSEXIT_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOABUSIVEESLINTDISABLE_NODE_TYPES: &[AstType] = &[];
pub const UNICORNNOABUSIVEESLINTDISABLE_ANY_NODE_TYPE: bool = true;

pub const UNICORNPREFERREFLECTAPPLY_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERREFLECTAPPLY_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERSTRUCTUREDCLONE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERSTRUCTUREDCLONE_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERDOMNODETEXTCONTENT_NODE_TYPES: &[AstType] = &[AstType::IdentifierName, AstType::IdentifierReference, AstType::StaticMemberExpression];
pub const UNICORNPREFERDOMNODETEXTCONTENT_ANY_NODE_TYPE: bool = false;

pub const UNICORNNEWFORBUILTINS_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::NewExpression];
pub const UNICORNNEWFORBUILTINS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOLENGTHASSLICEEND_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOLENGTHASSLICEEND_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOHEXESCAPE_NODE_TYPES: &[AstType] = &[AstType::RegExpLiteral, AstType::StringLiteral, AstType::TemplateLiteral];
pub const UNICORNNOHEXESCAPE_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERQUERYSELECTOR_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERQUERYSELECTOR_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERNEGATIVEINDEX_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERNEGATIVEINDEX_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERTYPEERROR_NODE_TYPES: &[AstType] = &[AstType::ThrowStatement];
pub const UNICORNPREFERTYPEERROR_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERBLOBREADINGMETHODS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERBLOBREADINGMETHODS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNONULL_NODE_TYPES: &[AstType] = &[AstType::NullLiteral];
pub const UNICORNNONULL_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERSTRINGSLICE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERSTRINGSLICE_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERMATHTRUNC_NODE_TYPES: &[AstType] = &[];
pub const UNICORNPREFERMATHTRUNC_ANY_NODE_TYPE: bool = true;

pub const UNICORNNOUNREADABLEARRAYDESTRUCTURING_NODE_TYPES: &[AstType] = &[AstType::ArrayPattern];
pub const UNICORNNOUNREADABLEARRAYDESTRUCTURING_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERCODEPOINT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERCODEPOINT_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOCONSOLESPACES_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOCONSOLESPACES_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERSTRINGSTARTSENDSWITH_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERSTRINGSTARTSENDSWITH_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOSINGLEPROMISEINPROMISEMETHODS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOSINGLEPROMISEINPROMISEMETHODS_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERNUMBERPROPERTIES_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::IdentifierReference];
pub const UNICORNPREFERNUMBERPROPERTIES_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOAWAITEXPRESSIONMEMBER_NODE_TYPES: &[AstType] = &[];
pub const UNICORNNOAWAITEXPRESSIONMEMBER_ANY_NODE_TYPE: bool = true;

pub const UNICORNNOINSTANCEOFARRAY_NODE_TYPES: &[AstType] = &[AstType::BinaryExpression];
pub const UNICORNNOINSTANCEOFARRAY_ANY_NODE_TYPE: bool = false;

pub const UNICORNERRORMESSAGE_NODE_TYPES: &[AstType] = &[];
pub const UNICORNERRORMESSAGE_ANY_NODE_TYPE: bool = true;

pub const UNICORNPREFERDOMNODEREMOVE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERDOMNODEREMOVE_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERADDEVENTLISTENER_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression];
pub const UNICORNPREFERADDEVENTLISTENER_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERDATENOW_NODE_TYPES: &[AstType] = &[AstType::AssignmentExpression, AstType::BinaryExpression, AstType::CallExpression, AstType::UnaryExpression];
pub const UNICORNPREFERDATENOW_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOUSELESSUNDEFINED_NODE_TYPES: &[AstType] = &[AstType::CallExpression, AstType::IdentifierReference];
pub const UNICORNNOUSELESSUNDEFINED_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOSTATICONLYCLASS_NODE_TYPES: &[AstType] = &[AstType::Class];
pub const UNICORNNOSTATICONLYCLASS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOUSELESSPROMISERESOLVEREJECT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOUSELESSPROMISERESOLVEREJECT_ANY_NODE_TYPE: bool = false;

pub const UNICORNESCAPECASE_NODE_TYPES: &[AstType] = &[AstType::RegExpLiteral, AstType::StringLiteral, AstType::TemplateLiteral];
pub const UNICORNESCAPECASE_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERLOGICALOPERATOROVERTERNARY_NODE_TYPES: &[AstType] = &[AstType::ConditionalExpression];
pub const UNICORNPREFERLOGICALOPERATOROVERTERNARY_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOOBJECTASDEFAULTPARAMETER_NODE_TYPES: &[AstType] = &[AstType::AssignmentPattern];
pub const UNICORNNOOBJECTASDEFAULTPARAMETER_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOUNNECESSARYARRAYFLATDEPTH_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOUNNECESSARYARRAYFLATDEPTH_ANY_NODE_TYPE: bool = false;

pub const NOUSELESSSPREADCONSTEVAL_NODE_TYPES: &[AstType] = &[];
pub const NOUSELESSSPREADCONSTEVAL_ANY_NODE_TYPE: bool = true;

pub const NOUSELESSSPREADMOD_NODE_TYPES: &[AstType] = &[AstType::ArrayExpression, AstType::ObjectExpression];
pub const NOUSELESSSPREADMOD_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERDOMNODEDATASET_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERDOMNODEDATASET_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOUNREADABLEIIFE_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOUNREADABLEIIFE_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERMODERNDOMAPIS_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERMODERNDOMAPIS_ANY_NODE_TYPE: bool = false;

pub const UNICORNNOARRAYFOREACH_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNNOARRAYFOREACH_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERGLOBALTHIS_NODE_TYPES: &[AstType] = &[AstType::IdentifierReference];
pub const UNICORNPREFERGLOBALTHIS_ANY_NODE_TYPE: bool = false;

pub const UNICORNPREFERREGEXPTEST_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNPREFERREGEXPTEST_ANY_NODE_TYPE: bool = false;

pub const UNICORNREQUIRENUMBERTOFIXEDDIGITSARGUMENT_NODE_TYPES: &[AstType] = &[AstType::CallExpression];
pub const UNICORNREQUIRENUMBERTOFIXEDDIGITSARGUMENT_ANY_NODE_TYPE: bool = false;

/// Get node types for a rule by its struct name
pub fn get_node_types(rule_name: &str) -> &'static [AstType] {
    match rule_name {
        "PromiseFunctionAsync" => TYPESCRIPTPROMISEFUNCTIONASYNC_NODE_TYPES,
        "NoFloatingPromises" => TYPESCRIPTNOFLOATINGPROMISES_NODE_TYPES,
        "NoUnnecessaryParameterPropertyAssignment" => TYPESCRIPTNOUNNECESSARYPARAMETERPROPERTYASSIGNMENT_NODE_TYPES,
        "NoUnnecessaryBooleanLiteralCompare" => TYPESCRIPTNOUNNECESSARYBOOLEANLITERALCOMPARE_NODE_TYPES,
        "NonNullableTypeAssertionStyle" => TYPESCRIPTNONNULLABLETYPEASSERTIONSTYLE_NODE_TYPES,
        "NoNonNullAssertedOptionalChain" => TYPESCRIPTNONONNULLASSERTEDOPTIONALCHAIN_NODE_TYPES,
        "NoUnsafeCall" => TYPESCRIPTNOUNSAFECALL_NODE_TYPES,
        "PreferFunctionType" => TYPESCRIPTPREFERFUNCTIONTYPE_NODE_TYPES,
        "ExplicitFunctionReturnType" => TYPESCRIPTEXPLICITFUNCTIONRETURNTYPE_NODE_TYPES,
        "SwitchExhaustivenessCheck" => TYPESCRIPTSWITCHEXHAUSTIVENESSCHECK_NODE_TYPES,
        "NoNonNullAssertion" => TYPESCRIPTNONONNULLASSERTION_NODE_TYPES,
        "RequireArraySortCompare" => TYPESCRIPTREQUIREARRAYSORTCOMPARE_NODE_TYPES,
        "NoUnsafeMemberAccess" => TYPESCRIPTNOUNSAFEMEMBERACCESS_NODE_TYPES,
        "NoDuplicateTypeConstituents" => TYPESCRIPTNODUPLICATETYPECONSTITUENTS_NODE_TYPES,
        "PreferEnumInitializers" => TYPESCRIPTPREFERENUMINITIALIZERS_NODE_TYPES,
        "NoArrayDelete" => TYPESCRIPTNOARRAYDELETE_NODE_TYPES,
        "NoUselessEmptyExport" => TYPESCRIPTNOUSELESSEMPTYEXPORT_NODE_TYPES,
        "ReturnAwait" => TYPESCRIPTRETURNAWAIT_NODE_TYPES,
        "NoMisusedSpread" => TYPESCRIPTNOMISUSEDSPREAD_NODE_TYPES,
        "RelatedGetterSetterPairs" => TYPESCRIPTRELATEDGETTERSETTERPAIRS_NODE_TYPES,
        "ArrayType" => TYPESCRIPTARRAYTYPE_NODE_TYPES,
        "PreferReduceTypeParameter" => TYPESCRIPTPREFERREDUCETYPEPARAMETER_NODE_TYPES,
        "PreferLiteralEnumMember" => TYPESCRIPTPREFERLITERALENUMMEMBER_NODE_TYPES,
        "UseUnknownInCatchCallbackVariable" => TYPESCRIPTUSEUNKNOWNINCATCHCALLBACKVARIABLE_NODE_TYPES,
        "NoDuplicateEnumValues" => TYPESCRIPTNODUPLICATEENUMVALUES_NODE_TYPES,
        "NoMisusedPromises" => TYPESCRIPTNOMISUSEDPROMISES_NODE_TYPES,
        "PreferForOf" => TYPESCRIPTPREFERFOROF_NODE_TYPES,
        "ConsistentIndexedObjectStyle" => TYPESCRIPTCONSISTENTINDEXEDOBJECTSTYLE_NODE_TYPES,
        "NoUnsafeAssignment" => TYPESCRIPTNOUNSAFEASSIGNMENT_NODE_TYPES,
        "NoRequireImports" => TYPESCRIPTNOREQUIREIMPORTS_NODE_TYPES,
        "NoRedundantTypeConstituents" => TYPESCRIPTNOREDUNDANTTYPECONSTITUENTS_NODE_TYPES,
        "PreferNamespaceKeyword" => TYPESCRIPTPREFERNAMESPACEKEYWORD_NODE_TYPES,
        "NoConfusingNonNullAssertion" => TYPESCRIPTNOCONFUSINGNONNULLASSERTION_NODE_TYPES,
        "NoThisAlias" => TYPESCRIPTNOTHISALIAS_NODE_TYPES,
        "NoUnsafeEnumComparison" => TYPESCRIPTNOUNSAFEENUMCOMPARISON_NODE_TYPES,
        "NoVarRequires" => TYPESCRIPTNOVARREQUIRES_NODE_TYPES,
        "AdjacentOverloadSignatures" => TYPESCRIPTADJACENTOVERLOADSIGNATURES_NODE_TYPES,
        "UnboundMethod" => TYPESCRIPTUNBOUNDMETHOD_NODE_TYPES,
        "NoWrapperObjectTypes" => TYPESCRIPTNOWRAPPEROBJECTTYPES_NODE_TYPES,
        "NoExplicitAny" => TYPESCRIPTNOEXPLICITANY_NODE_TYPES,
        "NoImportTypeSideEffects" => TYPESCRIPTNOIMPORTTYPESIDEEFFECTS_NODE_TYPES,
        "NoUnsafeFunctionType" => TYPESCRIPTNOUNSAFEFUNCTIONTYPE_NODE_TYPES,
        "NoConfusingVoidExpression" => TYPESCRIPTNOCONFUSINGVOIDEXPRESSION_NODE_TYPES,
        "NoUnsafeDeclarationMerging" => TYPESCRIPTNOUNSAFEDECLARATIONMERGING_NODE_TYPES,
        "NoMixedEnums" => TYPESCRIPTNOMIXEDENUMS_NODE_TYPES,
        "NoUnnecessaryTypeAssertion" => TYPESCRIPTNOUNNECESSARYTYPEASSERTION_NODE_TYPES,
        "NoBaseToString" => TYPESCRIPTNOBASETOSTRING_NODE_TYPES,
        "ConsistentTypeImports" => TYPESCRIPTCONSISTENTTYPEIMPORTS_NODE_TYPES,
        "PreferTsExpectError" => TYPESCRIPTPREFERTSEXPECTERROR_NODE_TYPES,
        "RestrictPlusOperands" => TYPESCRIPTRESTRICTPLUSOPERANDS_NODE_TYPES,
        "NoImpliedEval" => TYPESCRIPTNOIMPLIEDEVAL_NODE_TYPES,
        "OnlyThrowError" => TYPESCRIPTONLYTHROWERROR_NODE_TYPES,
        "AwaitThenable" => TYPESCRIPTAWAITTHENABLE_NODE_TYPES,
        "PreferReturnThisType" => TYPESCRIPTPREFERRETURNTHISTYPE_NODE_TYPES,
        "PreferAsConst" => TYPESCRIPTPREFERASCONST_NODE_TYPES,
        "PreferPromiseRejectErrors" => TYPESCRIPTPREFERPROMISEREJECTERRORS_NODE_TYPES,
        "NoInferrableTypes" => TYPESCRIPTNOINFERRABLETYPES_NODE_TYPES,
        "ExplicitModuleBoundaryTypes" => TYPESCRIPTEXPLICITMODULEBOUNDARYTYPES_NODE_TYPES,
        "NoMisusedNew" => TYPESCRIPTNOMISUSEDNEW_NODE_TYPES,
        "NoDynamicDelete" => TYPESCRIPTNODYNAMICDELETE_NODE_TYPES,
        "NoNonNullAssertedNullishCoalescing" => TYPESCRIPTNONONNULLASSERTEDNULLISHCOALESCING_NODE_TYPES,
        "BanTypes" => TYPESCRIPTBANTYPES_NODE_TYPES,
        "NoUnsafeReturn" => TYPESCRIPTNOUNSAFERETURN_NODE_TYPES,
        "NoMeaninglessVoidOperator" => TYPESCRIPTNOMEANINGLESSVOIDOPERATOR_NODE_TYPES,
        "NoUnnecessaryTemplateExpression" => TYPESCRIPTNOUNNECESSARYTEMPLATEEXPRESSION_NODE_TYPES,
        "NoUnsafeUnaryMinus" => TYPESCRIPTNOUNSAFEUNARYMINUS_NODE_TYPES,
        "NoEmptyObjectType" => TYPESCRIPTNOEMPTYOBJECTTYPE_NODE_TYPES,
        "ConsistentTypeDefinitions" => TYPESCRIPTCONSISTENTTYPEDEFINITIONS_NODE_TYPES,
        "NoExtraneousClass" => TYPESCRIPTNOEXTRANEOUSCLASS_NODE_TYPES,
        "ConsistentGenericConstructors" => TYPESCRIPTCONSISTENTGENERICCONSTRUCTORS_NODE_TYPES,
        "NoNamespace" => TYPESCRIPTNONAMESPACE_NODE_TYPES,
        "RequireAwait" => TYPESCRIPTREQUIREAWAIT_NODE_TYPES,
        "RestrictTemplateExpressions" => TYPESCRIPTRESTRICTTEMPLATEEXPRESSIONS_NODE_TYPES,
        "BanTslintComment" => TYPESCRIPTBANTSLINTCOMMENT_NODE_TYPES,
        "NoExtraNonNullAssertion" => TYPESCRIPTNOEXTRANONNULLASSERTION_NODE_TYPES,
        "NoUnnecessaryTypeConstraint" => TYPESCRIPTNOUNNECESSARYTYPECONSTRAINT_NODE_TYPES,
        "NoUnnecessaryTypeArguments" => TYPESCRIPTNOUNNECESSARYTYPEARGUMENTS_NODE_TYPES,
        "NoUnsafeArgument" => TYPESCRIPTNOUNSAFEARGUMENT_NODE_TYPES,
        "NoForInArray" => TYPESCRIPTNOFORINARRAY_NODE_TYPES,
        "BanTsComment" => TYPESCRIPTBANTSCOMMENT_NODE_TYPES,
        "NoUnsafeTypeAssertion" => TYPESCRIPTNOUNSAFETYPEASSERTION_NODE_TYPES,
        "TripleSlashReference" => TYPESCRIPTTRIPLESLASHREFERENCE_NODE_TYPES,
        "NoEmptyInterface" => TYPESCRIPTNOEMPTYINTERFACE_NODE_TYPES,
        "RequireLocalTestContextForConcurrentSnapshots" => VITESTREQUIRELOCALTESTCONTEXTFORCONCURRENTSNAPSHOTS_NODE_TYPES,
        "PreferToBeTruthy" => VITESTPREFERTOBETRUTHY_NODE_TYPES,
        "NoConditionalTests" => VITESTNOCONDITIONALTESTS_NODE_TYPES,
        "NoImportNodeTest" => VITESTNOIMPORTNODETEST_NODE_TYPES,
        "PreferToBeFalsy" => VITESTPREFERTOBEFALSY_NODE_TYPES,
        "PreferToBeObject" => VITESTPREFERTOBEOBJECT_NODE_TYPES,
        "NoReturnInFinally" => PROMISENORETURNINFINALLY_NODE_TYPES,
        "PreferCatch" => PROMISEPREFERCATCH_NODE_TYPES,
        "NoNesting" => PROMISENONESTING_NODE_TYPES,
        "NoNewStatics" => PROMISENONEWSTATICS_NODE_TYPES,
        "SpecOnly" => PROMISESPECONLY_NODE_TYPES,
        "NoCallbackInPromise" => PROMISENOCALLBACKINPROMISE_NODE_TYPES,
        "AvoidNew" => PROMISEAVOIDNEW_NODE_TYPES,
        "NoReturnWrap" => PROMISENORETURNWRAP_NODE_TYPES,
        "NoPromiseInCallback" => PROMISENOPROMISEINCALLBACK_NODE_TYPES,
        "ValidParams" => PROMISEVALIDPARAMS_NODE_TYPES,
        "PreferAwaitToCallbacks" => PROMISEPREFERAWAITTOCALLBACKS_NODE_TYPES,
        "ParamNames" => PROMISEPARAMNAMES_NODE_TYPES,
        "CatchOrReturn" => PROMISECATCHORRETURN_NODE_TYPES,
        "PreferAwaitToThen" => PROMISEPREFERAWAITTOTHEN_NODE_TYPES,
        "PreferHooksInOrder" => JESTPREFERHOOKSINORDER_NODE_TYPES,
        "NoHooks" => JESTNOHOOKS_NODE_TYPES,
        "PreferEqualityMatcher" => JESTPREFEREQUALITYMATCHER_NODE_TYPES,
        "ValidExpect" => JESTVALIDEXPECT_NODE_TYPES,
        "NoConditionalInTest" => JESTNOCONDITIONALINTEST_NODE_TYPES,
        "ExpectExpect" => JESTEXPECTEXPECT_NODE_TYPES,
        "PreferCalledWith" => JESTPREFERCALLEDWITH_NODE_TYPES,
        "NoLargeSnapshots" => JESTNOLARGESNAPSHOTS_NODE_TYPES,
        "NoTestReturnStatement" => JESTNOTESTRETURNSTATEMENT_NODE_TYPES,
        "NoMocksImport" => JESTNOMOCKSIMPORT_NODE_TYPES,
        "NoRestrictedMatchers" => JESTNORESTRICTEDMATCHERS_NODE_TYPES,
        "NoDuplicateHooks" => JESTNODUPLICATEHOOKS_NODE_TYPES,
        "NoCommentedOutTests" => JESTNOCOMMENTEDOUTTESTS_NODE_TYPES,
        "PreferStrictEqual" => JESTPREFERSTRICTEQUAL_NODE_TYPES,
        "PreferComparisonMatcher" => JESTPREFERCOMPARISONMATCHER_NODE_TYPES,
        "NoInterpolationInSnapshots" => JESTNOINTERPOLATIONINSNAPSHOTS_NODE_TYPES,
        "PreferToContain" => JESTPREFERTOCONTAIN_NODE_TYPES,
        "PreferTodo" => JESTPREFERTODO_NODE_TYPES,
        "Mod" => NOSTANDALONEEXPECTMOD_NODE_TYPES,
        "NoDisabledTests" => JESTNODISABLEDTESTS_NODE_TYPES,
        "NoJasmineGlobals" => JESTNOJASMINEGLOBALS_NODE_TYPES,
        "Mod" => PREFERLOWERCASETITLEMOD_NODE_TYPES,
        "PreferToHaveLength" => JESTPREFERTOHAVELENGTH_NODE_TYPES,
        "ValidTitle" => JESTVALIDTITLE_NODE_TYPES,
        "PreferHooksOnTop" => JESTPREFERHOOKSONTOP_NODE_TYPES,
        "PreferExpectResolves" => JESTPREFEREXPECTRESOLVES_NODE_TYPES,
        "PreferJestMocked" => JESTPREFERJESTMOCKED_NODE_TYPES,
        "ValidDescribeCallback" => JESTVALIDDESCRIBECALLBACK_NODE_TYPES,
        "PreferMockPromiseShorthand" => JESTPREFERMOCKPROMISESHORTHAND_NODE_TYPES,
        "NoConfusingSetTimeout" => JESTNOCONFUSINGSETTIMEOUT_NODE_TYPES,
        "NoTestPrefixes" => JESTNOTESTPREFIXES_NODE_TYPES,
        "NoConditionalExpect" => JESTNOCONDITIONALEXPECT_NODE_TYPES,
        "NoFocusedTests" => JESTNOFOCUSEDTESTS_NODE_TYPES,
        "PreferEach" => JESTPREFEREACH_NODE_TYPES,
        "NoIdenticalTitle" => JESTNOIDENTICALTITLE_NODE_TYPES,
        "NoRestrictedJestMethods" => JESTNORESTRICTEDJESTMETHODS_NODE_TYPES,
        "PreferToBe" => JESTPREFERTOBE_NODE_TYPES,
        "MaxExpects" => JESTMAXEXPECTS_NODE_TYPES,
        "NoDeprecatedFunctions" => JESTNODEPRECATEDFUNCTIONS_NODE_TYPES,
        "RequireHook" => JESTREQUIREHOOK_NODE_TYPES,
        "MaxNestedDescribe" => JESTMAXNESTEDDESCRIBE_NODE_TYPES,
        "RequireToThrowMessage" => JESTREQUIRETOTHROWMESSAGE_NODE_TYPES,
        "ConsistentTestIt" => JESTCONSISTENTTESTIT_NODE_TYPES,
        "NoDoneCallback" => JESTNODONECALLBACK_NODE_TYPES,
        "RequireTopLevelDescribe" => JESTREQUIRETOPLEVELDESCRIBE_NODE_TYPES,
        "NoAliasMethods" => JESTNOALIASMETHODS_NODE_TYPES,
        "NoExport" => JESTNOEXPORT_NODE_TYPES,
        "PreferSpyOn" => JESTPREFERSPYON_NODE_TYPES,
        "NoUntypedMockFactory" => JESTNOUNTYPEDMOCKFACTORY_NODE_TYPES,
        "Lang" => JSXA11YLANG_NODE_TYPES,
        "PreferTagOverRole" => JSXA11YPREFERTAGOVERROLE_NODE_TYPES,
        "AriaActivedescendantHasTabindex" => JSXA11YARIAACTIVEDESCENDANTHASTABINDEX_NODE_TYPES,
        "NoRedundantRoles" => JSXA11YNOREDUNDANTROLES_NODE_TYPES,
        "ImgRedundantAlt" => JSXA11YIMGREDUNDANTALT_NODE_TYPES,
        "AriaProps" => JSXA11YARIAPROPS_NODE_TYPES,
        "AriaUnsupportedElements" => JSXA11YARIAUNSUPPORTEDELEMENTS_NODE_TYPES,
        "RoleHasRequiredAriaProps" => JSXA11YROLEHASREQUIREDARIAPROPS_NODE_TYPES,
        "ClickEventsHaveKeyEvents" => JSXA11YCLICKEVENTSHAVEKEYEVENTS_NODE_TYPES,
        "TabindexNoPositive" => JSXA11YTABINDEXNOPOSITIVE_NODE_TYPES,
        "NoAutofocus" => JSXA11YNOAUTOFOCUS_NODE_TYPES,
        "LabelHasAssociatedControl" => JSXA11YLABELHASASSOCIATEDCONTROL_NODE_TYPES,
        "MediaHasCaption" => JSXA11YMEDIAHASCAPTION_NODE_TYPES,
        "AutocompleteValid" => JSXA11YAUTOCOMPLETEVALID_NODE_TYPES,
        "Scope" => JSXA11YSCOPE_NODE_TYPES,
        "AriaRole" => JSXA11YARIAROLE_NODE_TYPES,
        "NoDistractingElements" => JSXA11YNODISTRACTINGELEMENTS_NODE_TYPES,
        "HeadingHasContent" => JSXA11YHEADINGHASCONTENT_NODE_TYPES,
        "HtmlHasLang" => JSXA11YHTMLHASLANG_NODE_TYPES,
        "IframeHasTitle" => JSXA11YIFRAMEHASTITLE_NODE_TYPES,
        "NoNoninteractiveTabindex" => JSXA11YNONONINTERACTIVETABINDEX_NODE_TYPES,
        "AnchorAmbiguousText" => JSXA11YANCHORAMBIGUOUSTEXT_NODE_TYPES,
        "AnchorHasContent" => JSXA11YANCHORHASCONTENT_NODE_TYPES,
        "AnchorIsValid" => JSXA11YANCHORISVALID_NODE_TYPES,
        "AltText" => JSXA11YALTTEXT_NODE_TYPES,
        "NoAccessKey" => JSXA11YNOACCESSKEY_NODE_TYPES,
        "MouseEventsHaveKeyEvents" => JSXA11YMOUSEEVENTSHAVEKEYEVENTS_NODE_TYPES,
        "RoleSupportsAriaProps" => JSXA11YROLESUPPORTSARIAPROPS_NODE_TYPES,
        "NoAriaHiddenOnFocusable" => JSXA11YNOARIAHIDDENONFOCUSABLE_NODE_TYPES,
        "NoNewRequire" => NODENONEWREQUIRE_NODE_TYPES,
        "NoExportsAssign" => NODENOEXPORTSASSIGN_NODE_TYPES,
        "GoogleFontDisplay" => NEXTJSGOOGLEFONTDISPLAY_NODE_TYPES,
        "NoUnwantedPolyfillio" => NEXTJSNOUNWANTEDPOLYFILLIO_NODE_TYPES,
        "NoCssTags" => NEXTJSNOCSSTAGS_NODE_TYPES,
        "NoHtmlLinkForPages" => NEXTJSNOHTMLLINKFORPAGES_NODE_TYPES,
        "NoStyledJsxInDocument" => NEXTJSNOSTYLEDJSXINDOCUMENT_NODE_TYPES,
        "NoDuplicateHead" => NEXTJSNODUPLICATEHEAD_NODE_TYPES,
        "NoPageCustomFont" => NEXTJSNOPAGECUSTOMFONT_NODE_TYPES,
        "NoHeadImportInDocument" => NEXTJSNOHEADIMPORTINDOCUMENT_NODE_TYPES,
        "NoBeforeInteractiveScriptOutsideDocument" => NEXTJSNOBEFOREINTERACTIVESCRIPTOUTSIDEDOCUMENT_NODE_TYPES,
        "GoogleFontPreconnect" => NEXTJSGOOGLEFONTPRECONNECT_NODE_TYPES,
        "NoSyncScripts" => NEXTJSNOSYNCSCRIPTS_NODE_TYPES,
        "NoHeadElement" => NEXTJSNOHEADELEMENT_NODE_TYPES,
        "NoImgElement" => NEXTJSNOIMGELEMENT_NODE_TYPES,
        "InlineScriptId" => NEXTJSINLINESCRIPTID_NODE_TYPES,
        "NextScriptForGa" => NEXTJSNEXTSCRIPTFORGA_NODE_TYPES,
        "NoScriptComponentInHead" => NEXTJSNOSCRIPTCOMPONENTINHEAD_NODE_TYPES,
        "NoTitleInDocumentHead" => NEXTJSNOTITLEINDOCUMENTHEAD_NODE_TYPES,
        "NoDocumentImportInPage" => NEXTJSNODOCUMENTIMPORTINPAGE_NODE_TYPES,
        "NoAssignModuleVariable" => NEXTJSNOASSIGNMODULEVARIABLE_NODE_TYPES,
        "NoTypos" => NEXTJSNOTYPOS_NODE_TYPES,
        "NoAsyncClientComponent" => NEXTJSNOASYNCCLIENTCOMPONENT_NODE_TYPES,
        "ExportsLast" => IMPORTEXPORTSLAST_NODE_TYPES,
        "Export" => IMPORTEXPORT_NODE_TYPES,
        "NoNamedDefault" => IMPORTNONAMEDDEFAULT_NODE_TYPES,
        "NoAnonymousDefaultExport" => IMPORTNOANONYMOUSDEFAULTEXPORT_NODE_TYPES,
        "NoWebpackLoaderSyntax" => IMPORTNOWEBPACKLOADERSYNTAX_NODE_TYPES,
        "Named" => IMPORTNAMED_NODE_TYPES,
        "First" => IMPORTFIRST_NODE_TYPES,
        "NoDuplicates" => IMPORTNODUPLICATES_NODE_TYPES,
        "NoDefaultExport" => IMPORTNODEFAULTEXPORT_NODE_TYPES,
        "ConsistentTypeSpecifierStyle" => IMPORTCONSISTENTTYPESPECIFIERSTYLE_NODE_TYPES,
        "GroupExports" => IMPORTGROUPEXPORTS_NODE_TYPES,
        "NoNamedAsDefaultMember" => IMPORTNONAMEDASDEFAULTMEMBER_NODE_TYPES,
        "Namespace" => IMPORTNAMESPACE_NODE_TYPES,
        "NoAbsolutePath" => IMPORTNOABSOLUTEPATH_NODE_TYPES,
        "Extensions" => IMPORTEXTENSIONS_NODE_TYPES,
        "PreferDefaultExport" => IMPORTPREFERDEFAULTEXPORT_NODE_TYPES,
        "NoMutableExports" => IMPORTNOMUTABLEEXPORTS_NODE_TYPES,
        "NoCycle" => IMPORTNOCYCLE_NODE_TYPES,
        "NoCommonjs" => IMPORTNOCOMMONJS_NODE_TYPES,
        "NoDynamicRequire" => IMPORTNODYNAMICREQUIRE_NODE_TYPES,
        "NoNamespace" => IMPORTNONAMESPACE_NODE_TYPES,
        "NoAmd" => IMPORTNOAMD_NODE_TYPES,
        "NoEmptyNamedBlocks" => IMPORTNOEMPTYNAMEDBLOCKS_NODE_TYPES,
        "NoSelfImport" => IMPORTNOSELFIMPORT_NODE_TYPES,
        "NoNamedAsDefault" => IMPORTNONAMEDASDEFAULT_NODE_TYPES,
        "Unambiguous" => IMPORTUNAMBIGUOUS_NODE_TYPES,
        "NoUnassignedImport" => IMPORTNOUNASSIGNEDIMPORT_NODE_TYPES,
        "MaxDependencies" => IMPORTMAXDEPENDENCIES_NODE_TYPES,
        "Default" => IMPORTDEFAULT_NODE_TYPES,
        "EmptyTags" => JSDOCEMPTYTAGS_NODE_TYPES,
        "ImplementsOnClasses" => JSDOCIMPLEMENTSONCLASSES_NODE_TYPES,
        "RequirePropertyType" => JSDOCREQUIREPROPERTYTYPE_NODE_TYPES,
        "CheckAccess" => JSDOCCHECKACCESS_NODE_TYPES,
        "RequireReturns" => JSDOCREQUIRERETURNS_NODE_TYPES,
        "RequireProperty" => JSDOCREQUIREPROPERTY_NODE_TYPES,
        "RequireReturnsDescription" => JSDOCREQUIRERETURNSDESCRIPTION_NODE_TYPES,
        "CheckPropertyNames" => JSDOCCHECKPROPERTYNAMES_NODE_TYPES,
        "RequireReturnsType" => JSDOCREQUIRERETURNSTYPE_NODE_TYPES,
        "RequireYields" => JSDOCREQUIREYIELDS_NODE_TYPES,
        "RequireParamDescription" => JSDOCREQUIREPARAMDESCRIPTION_NODE_TYPES,
        "RequireParamName" => JSDOCREQUIREPARAMNAME_NODE_TYPES,
        "NoDefaults" => JSDOCNODEFAULTS_NODE_TYPES,
        "RequirePropertyDescription" => JSDOCREQUIREPROPERTYDESCRIPTION_NODE_TYPES,
        "RequireParam" => JSDOCREQUIREPARAM_NODE_TYPES,
        "RequirePropertyName" => JSDOCREQUIREPROPERTYNAME_NODE_TYPES,
        "CheckTagNames" => JSDOCCHECKTAGNAMES_NODE_TYPES,
        "RequireParamType" => JSDOCREQUIREPARAMTYPE_NODE_TYPES,
        "NoRestrictedGlobals" => ESLINTNORESTRICTEDGLOBALS_NODE_TYPES,
        "NoNewFunc" => ESLINTNONEWFUNC_NODE_TYPES,
        "RequireYield" => ESLINTREQUIREYIELD_NODE_TYPES,
        "NoUndef" => ESLINTNOUNDEF_NODE_TYPES,
        "GetterReturn" => ESLINTGETTERRETURN_NODE_TYPES,
        "Radix" => ESLINTRADIX_NODE_TYPES,
        "NoGlobalAssign" => ESLINTNOGLOBALASSIGN_NODE_TYPES,
        "NoUnusedExpressions" => ESLINTNOUNUSEDEXPRESSIONS_NODE_TYPES,
        "IdLength" => ESLINTIDLENGTH_NODE_TYPES,
        "NoSetterReturn" => ESLINTNOSETTERRETURN_NODE_TYPES,
        "NoConstantCondition" => ESLINTNOCONSTANTCONDITION_NODE_TYPES,
        "Usage" => NOUNUSEDVARSUSAGE_NODE_TYPES,
        "BindingPattern" => NOUNUSEDVARSBINDINGPATTERN_NODE_TYPES,
        "Diagnostic" => NOUNUSEDVARSDIAGNOSTIC_NODE_TYPES,
        "Symbol" => NOUNUSEDVARSSYMBOL_NODE_TYPES,
        "Options" => NOUNUSEDVARSOPTIONS_NODE_TYPES,
        "Allowed" => NOUNUSEDVARSALLOWED_NODE_TYPES,
        "Ignored" => NOUNUSEDVARSIGNORED_NODE_TYPES,
        "Mod" => NOUNUSEDVARSMOD_NODE_TYPES,
        "NoDupeClassMembers" => ESLINTNODUPECLASSMEMBERS_NODE_TYPES,
        "PreferObjectSpread" => ESLINTPREFEROBJECTSPREAD_NODE_TYPES,
        "Yoda" => ESLINTYODA_NODE_TYPES,
        "FuncNames" => ESLINTFUNCNAMES_NODE_TYPES,
        "NoLonelyIf" => ESLINTNOLONELYIF_NODE_TYPES,
        "NoPlusplus" => ESLINTNOPLUSPLUS_NODE_TYPES,
        "NoIrregularWhitespace" => ESLINTNOIRREGULARWHITESPACE_NODE_TYPES,
        "NoArrayConstructor" => ESLINTNOARRAYCONSTRUCTOR_NODE_TYPES,
        "ValidTypeof" => ESLINTVALIDTYPEOF_NODE_TYPES,
        "NoWith" => ESLINTNOWITH_NODE_TYPES,
        "NoSelfCompare" => ESLINTNOSELFCOMPARE_NODE_TYPES,
        "MaxLinesPerFunction" => ESLINTMAXLINESPERFUNCTION_NODE_TYPES,
        "DefaultParamLast" => ESLINTDEFAULTPARAMLAST_NODE_TYPES,
        "UseIsnan" => ESLINTUSEISNAN_NODE_TYPES,
        "NoObjectConstructor" => ESLINTNOOBJECTCONSTRUCTOR_NODE_TYPES,
        "SortVars" => ESLINTSORTVARS_NODE_TYPES,
        "NoUnusedPrivateClassMembers" => ESLINTNOUNUSEDPRIVATECLASSMEMBERS_NODE_TYPES,
        "NoThrowLiteral" => ESLINTNOTHROWLITERAL_NODE_TYPES,
        "NoMultiStr" => ESLINTNOMULTISTR_NODE_TYPES,
        "NoEmptyPattern" => ESLINTNOEMPTYPATTERN_NODE_TYPES,
        "NoDeleteVar" => ESLINTNODELETEVAR_NODE_TYPES,
        "NoLossOfPrecision" => ESLINTNOLOSSOFPRECISION_NODE_TYPES,
        "SortImports" => ESLINTSORTIMPORTS_NODE_TYPES,
        "NoLabels" => ESLINTNOLABELS_NODE_TYPES,
        "PreferSpread" => ESLINTPREFERSPREAD_NODE_TYPES,
        "NoSparseArrays" => ESLINTNOSPARSEARRAYS_NODE_TYPES,
        "MaxParams" => ESLINTMAXPARAMS_NODE_TYPES,
        "NoEmptyStaticBlock" => ESLINTNOEMPTYSTATICBLOCK_NODE_TYPES,
        "NoVar" => ESLINTNOVAR_NODE_TYPES,
        "Curly" => ESLINTCURLY_NODE_TYPES,
        "NoScriptUrl" => ESLINTNOSCRIPTURL_NODE_TYPES,
        "SymbolDescription" => ESLINTSYMBOLDESCRIPTION_NODE_TYPES,
        "NoNestedTernary" => ESLINTNONESTEDTERNARY_NODE_TYPES,
        "NoUselessConcat" => ESLINTNOUSELESSCONCAT_NODE_TYPES,
        "NoIterator" => ESLINTNOITERATOR_NODE_TYPES,
        "NoDebugger" => ESLINTNODEBUGGER_NODE_TYPES,
        "NoImportAssign" => ESLINTNOIMPORTASSIGN_NODE_TYPES,
        "NoUnsafeFinally" => ESLINTNOUNSAFEFINALLY_NODE_TYPES,
        "NoTemplateCurlyInString" => ESLINTNOTEMPLATECURLYINSTRING_NODE_TYPES,
        "NoFuncAssign" => ESLINTNOFUNCASSIGN_NODE_TYPES,
        "NoEmptyCharacterClass" => ESLINTNOEMPTYCHARACTERCLASS_NODE_TYPES,
        "NoExtraBind" => ESLINTNOEXTRABIND_NODE_TYPES,
        "SortKeys" => ESLINTSORTKEYS_NODE_TYPES,
        "NoRegexSpaces" => ESLINTNOREGEXSPACES_NODE_TYPES,
        "NoClassAssign" => ESLINTNOCLASSASSIGN_NODE_TYPES,
        "NoEval" => ESLINTNOEVAL_NODE_TYPES,
        "Eqeqeq" => ESLINTEQEQEQ_NODE_TYPES,
        "NoNegatedCondition" => ESLINTNONEGATEDCONDITION_NODE_TYPES,
        "NoUnusedLabels" => ESLINTNOUNUSEDLABELS_NODE_TYPES,
        "PreferNumericLiterals" => ESLINTPREFERNUMERICLITERALS_NODE_TYPES,
        "NoUselessConstructor" => ESLINTNOUSELESSCONSTRUCTOR_NODE_TYPES,
        "BlockScopedVar" => ESLINTBLOCKSCOPEDVAR_NODE_TYPES,
        "NoBitwise" => ESLINTNOBITWISE_NODE_TYPES,
        "NoEqNull" => ESLINTNOEQNULL_NODE_TYPES,
        "NoCaseDeclarations" => ESLINTNOCASEDECLARATIONS_NODE_TYPES,
        "NoNonoctalDecimalEscape" => ESLINTNONONOCTALDECIMALESCAPE_NODE_TYPES,
        "NewCap" => ESLINTNEWCAP_NODE_TYPES,
        "NoAwaitInLoop" => ESLINTNOAWAITINLOOP_NODE_TYPES,
        "NoUnassignedVars" => ESLINTNOUNASSIGNEDVARS_NODE_TYPES,
        "NoEmpty" => ESLINTNOEMPTY_NODE_TYPES,
        "NoControlRegex" => ESLINTNOCONTROLREGEX_NODE_TYPES,
        "DefaultCase" => ESLINTDEFAULTCASE_NODE_TYPES,
        "NoDuplicateCase" => ESLINTNODUPLICATECASE_NODE_TYPES,
        "NoElseReturn" => ESLINTNOELSERETURN_NODE_TYPES,
        "NoRedeclare" => ESLINTNOREDECLARE_NODE_TYPES,
        "OperatorAssignment" => ESLINTOPERATORASSIGNMENT_NODE_TYPES,
        "NoMagicNumbers" => ESLINTNOMAGICNUMBERS_NODE_TYPES,
        "NoUselessRename" => ESLINTNOUSELESSRENAME_NODE_TYPES,
        "NoExtendNative" => ESLINTNOEXTENDNATIVE_NODE_TYPES,
        "ForDirection" => ESLINTFORDIRECTION_NODE_TYPES,
        "NoInnerDeclarations" => ESLINTNOINNERDECLARATIONS_NODE_TYPES,
        "NoMultiAssign" => ESLINTNOMULTIASSIGN_NODE_TYPES,
        "NoRestrictedImports" => ESLINTNORESTRICTEDIMPORTS_NODE_TYPES,
        "NoConstAssign" => ESLINTNOCONSTASSIGN_NODE_TYPES,
        "GroupedAccessorPairs" => ESLINTGROUPEDACCESSORPAIRS_NODE_TYPES,
        "NoDuplicateImports" => ESLINTNODUPLICATEIMPORTS_NODE_TYPES,
        "NoVoid" => ESLINTNOVOID_NODE_TYPES,
        "NoUnneededTernary" => ESLINTNOUNNEEDEDTERNARY_NODE_TYPES,
        "NoCompareNegZero" => ESLINTNOCOMPARENEGZERO_NODE_TYPES,
        "NoNew" => ESLINTNONEW_NODE_TYPES,
        "PreferPromiseRejectErrors" => ESLINTPREFERPROMISEREJECTERRORS_NODE_TYPES,
        "InitDeclarations" => ESLINTINITDECLARATIONS_NODE_TYPES,
        "PreferRestParams" => ESLINTPREFERRESTPARAMS_NODE_TYPES,
        "NoExtraLabel" => ESLINTNOEXTRALABEL_NODE_TYPES,
        "PreferExponentiationOperator" => ESLINTPREFEREXPONENTIATIONOPERATOR_NODE_TYPES,
        "NoCondAssign" => ESLINTNOCONDASSIGN_NODE_TYPES,
        "NoUselessCall" => ESLINTNOUSELESSCALL_NODE_TYPES,
        "NoAlert" => ESLINTNOALERT_NODE_TYPES,
        "NoDupeKeys" => ESLINTNODUPEKEYS_NODE_TYPES,
        "NoUnexpectedMultiline" => ESLINTNOUNEXPECTEDMULTILINE_NODE_TYPES,
        "NoUselessBackreference" => ESLINTNOUSELESSBACKREFERENCE_NODE_TYPES,
        "NoConstructorReturn" => ESLINTNOCONSTRUCTORRETURN_NODE_TYPES,
        "NoCaller" => ESLINTNOCALLER_NODE_TYPES,
        "MaxDepth" => ESLINTMAXDEPTH_NODE_TYPES,
        "NoPrototypeBuiltins" => ESLINTNOPROTOTYPEBUILTINS_NODE_TYPES,
        "UnicodeBom" => ESLINTUNICODEBOM_NODE_TYPES,
        "DefaultCaseLast" => ESLINTDEFAULTCASELAST_NODE_TYPES,
        "NoUnreachable" => ESLINTNOUNREACHABLE_NODE_TYPES,
        "PreferDestructuring" => ESLINTPREFERDESTRUCTURING_NODE_TYPES,
        "NoNewNativeNonconstructor" => ESLINTNONEWNATIVENONCONSTRUCTOR_NODE_TYPES,
        "NoUnsafeNegation" => ESLINTNOUNSAFENEGATION_NODE_TYPES,
        "NoUselessEscape" => ESLINTNOUSELESSESCAPE_NODE_TYPES,
        "NoLoneBlocks" => ESLINTNOLONEBLOCKS_NODE_TYPES,
        "ReturnChecker" => ARRAYCALLBACKRETURNRETURNCHECKER_NODE_TYPES,
        "Mod" => ARRAYCALLBACKRETURNMOD_NODE_TYPES,
        "NoConstantBinaryExpression" => ESLINTNOCONSTANTBINARYEXPRESSION_NODE_TYPES,
        "VarsOnTop" => ESLINTVARSONTOP_NODE_TYPES,
        "MaxLines" => ESLINTMAXLINES_NODE_TYPES,
        "NoUndefined" => ESLINTNOUNDEFINED_NODE_TYPES,
        "NoTernary" => ESLINTNOTERNARY_NODE_TYPES,
        "NoObjCalls" => ESLINTNOOBJCALLS_NODE_TYPES,
        "NoReturnAssign" => ESLINTNORETURNASSIGN_NODE_TYPES,
        "NoShadowRestrictedNames" => ESLINTNOSHADOWRESTRICTEDNAMES_NODE_TYPES,
        "MaxNestedCallbacks" => ESLINTMAXNESTEDCALLBACKS_NODE_TYPES,
        "NoAsyncPromiseExecutor" => ESLINTNOASYNCPROMISEEXECUTOR_NODE_TYPES,
        "NoConsole" => ESLINTNOCONSOLE_NODE_TYPES,
        "NoFallthrough" => ESLINTNOFALLTHROUGH_NODE_TYPES,
        "NoEmptyFunction" => ESLINTNOEMPTYFUNCTION_NODE_TYPES,
        "NoUselessCatch" => ESLINTNOUSELESSCATCH_NODE_TYPES,
        "NoNewWrappers" => ESLINTNONEWWRAPPERS_NODE_TYPES,
        "RequireAwait" => ESLINTREQUIREAWAIT_NODE_TYPES,
        "NoThisBeforeSuper" => ESLINTNOTHISBEFORESUPER_NODE_TYPES,
        "NoExtraBooleanCast" => ESLINTNOEXTRABOOLEANCAST_NODE_TYPES,
        "NoInvalidRegexp" => ESLINTNOINVALIDREGEXP_NODE_TYPES,
        "FuncStyle" => ESLINTFUNCSTYLE_NODE_TYPES,
        "NoLabelVar" => ESLINTNOLABELVAR_NODE_TYPES,
        "NoProto" => ESLINTNOPROTO_NODE_TYPES,
        "NoUnsafeOptionalChaining" => ESLINTNOUNSAFEOPTIONALCHAINING_NODE_TYPES,
        "GuardForIn" => ESLINTGUARDFORIN_NODE_TYPES,
        "PreferObjectHasOwn" => ESLINTPREFEROBJECTHASOWN_NODE_TYPES,
        "NoDivRegex" => ESLINTNODIVREGEX_NODE_TYPES,
        "NoExAssign" => ESLINTNOEXASSIGN_NODE_TYPES,
        "MaxClassesPerFile" => ESLINTMAXCLASSESPERFILE_NODE_TYPES,
        "NoContinue" => ESLINTNOCONTINUE_NODE_TYPES,
        "NoDupeElseIf" => ESLINTNODUPEELSEIF_NODE_TYPES,
        "ArrowBodyStyle" => ESLINTARROWBODYSTYLE_NODE_TYPES,
        "NoSelfAssign" => ESLINTNOSELFASSIGN_NODE_TYPES,
        "JsxNoNewObjectAsProp" => REACTPERFJSXNONEWOBJECTASPROP_NODE_TYPES,
        "JsxNoNewArrayAsProp" => REACTPERFJSXNONEWARRAYASPROP_NODE_TYPES,
        "JsxNoNewFunctionAsProp" => REACTPERFJSXNONEWFUNCTIONASPROP_NODE_TYPES,
        "JsxNoJsxAsProp" => REACTPERFJSXNOJSXASPROP_NODE_TYPES,
        "MisrefactoredAssignOp" => OXCMISREFACTOREDASSIGNOP_NODE_TYPES,
        "OnlyUsedInRecursion" => OXCONLYUSEDINRECURSION_NODE_TYPES,
        "NoAccumulatingSpread" => OXCNOACCUMULATINGSPREAD_NODE_TYPES,
        "NoOptionalChaining" => OXCNOOPTIONALCHAINING_NODE_TYPES,
        "NoConstEnum" => OXCNOCONSTENUM_NODE_TYPES,
        "BadReplaceAllArg" => OXCBADREPLACEALLARG_NODE_TYPES,
        "BadArrayMethodOnArguments" => OXCBADARRAYMETHODONARGUMENTS_NODE_TYPES,
        "BadCharAtComparison" => OXCBADCHARATCOMPARISON_NODE_TYPES,
        "NoBarrelFile" => OXCNOBARRELFILE_NODE_TYPES,
        "NoRestSpreadProperties" => OXCNORESTSPREADPROPERTIES_NODE_TYPES,
        "DoubleComparisons" => OXCDOUBLECOMPARISONS_NODE_TYPES,
        "BadBitwiseOperator" => OXCBADBITWISEOPERATOR_NODE_TYPES,
        "MissingThrow" => OXCMISSINGTHROW_NODE_TYPES,
        "ApproxConstant" => OXCAPPROXCONSTANT_NODE_TYPES,
        "BadMinMaxFunc" => OXCBADMINMAXFUNC_NODE_TYPES,
        "BadComparisonSequence" => OXCBADCOMPARISONSEQUENCE_NODE_TYPES,
        "UninvokedArrayCallback" => OXCUNINVOKEDARRAYCALLBACK_NODE_TYPES,
        "ErasingOp" => OXCERASINGOP_NODE_TYPES,
        "NumberArgOutOfRange" => OXCNUMBERARGOUTOFRANGE_NODE_TYPES,
        "ConstComparisons" => OXCCONSTCOMPARISONS_NODE_TYPES,
        "NoAsyncEndpointHandlers" => OXCNOASYNCENDPOINTHANDLERS_NODE_TYPES,
        "BadObjectLiteralComparison" => OXCBADOBJECTLITERALCOMPARISON_NODE_TYPES,
        "NoAsyncAwait" => OXCNOASYNCAWAIT_NODE_TYPES,
        "NoMapSpread" => OXCNOMAPSPREAD_NODE_TYPES,
        "JsxNoScriptUrl" => REACTJSXNOSCRIPTURL_NODE_TYPES,
        "CheckedRequiresOnchangeOrReadonly" => REACTCHECKEDREQUIRESONCHANGEORREADONLY_NODE_TYPES,
        "RulesOfHooks" => REACTRULESOFHOOKS_NODE_TYPES,
        "JsxNoUndef" => REACTJSXNOUNDEF_NODE_TYPES,
        "JsxFragments" => REACTJSXFRAGMENTS_NODE_TYPES,
        "JsxCurlyBracePresence" => REACTJSXCURLYBRACEPRESENCE_NODE_TYPES,
        "RequireRenderReturn" => REACTREQUIRERENDERRETURN_NODE_TYPES,
        "NoStringRefs" => REACTNOSTRINGREFS_NODE_TYPES,
        "JsxNoDuplicateProps" => REACTJSXNODUPLICATEPROPS_NODE_TYPES,
        "StylePropObject" => REACTSTYLEPROPOBJECT_NODE_TYPES,
        "NoChildrenProp" => REACTNOCHILDRENPROP_NODE_TYPES,
        "ExhaustiveDeps" => REACTEXHAUSTIVEDEPS_NODE_TYPES,
        "NoDirectMutationState" => REACTNODIRECTMUTATIONSTATE_NODE_TYPES,
        "JsxPropsNoSpreadMulti" => REACTJSXPROPSNOSPREADMULTI_NODE_TYPES,
        "JsxBooleanValue" => REACTJSXBOOLEANVALUE_NODE_TYPES,
        "IframeMissingSandbox" => REACTIFRAMEMISSINGSANDBOX_NODE_TYPES,
        "NoDangerWithChildren" => REACTNODANGERWITHCHILDREN_NODE_TYPES,
        "JsxNoTargetBlank" => REACTJSXNOTARGETBLANK_NODE_TYPES,
        "NoIsMounted" => REACTNOISMOUNTED_NODE_TYPES,
        "PreferEs6Class" => REACTPREFERES6CLASS_NODE_TYPES,
        "ForbidElements" => REACTFORBIDELEMENTS_NODE_TYPES,
        "JsxFilenameExtension" => REACTJSXFILENAMEEXTENSION_NODE_TYPES,
        "VoidDomElementsNoChildren" => REACTVOIDDOMELEMENTSNOCHILDREN_NODE_TYPES,
        "NoSetState" => REACTNOSETSTATE_NODE_TYPES,
        "ButtonHasType" => REACTBUTTONHASTYPE_NODE_TYPES,
        "NoUnescapedEntities" => REACTNOUNESCAPEDENTITIES_NODE_TYPES,
        "NoFindDomNode" => REACTNOFINDDOMNODE_NODE_TYPES,
        "ReactInJsxScope" => REACTREACTINJSXSCOPE_NODE_TYPES,
        "SelfClosingComp" => REACTSELFCLOSINGCOMP_NODE_TYPES,
        "JsxNoUselessFragment" => REACTJSXNOUSELESSFRAGMENT_NODE_TYPES,
        "ForwardRefUsesRef" => REACTFORWARDREFUSESREF_NODE_TYPES,
        "JsxKey" => REACTJSXKEY_NODE_TYPES,
        "NoNamespace" => REACTNONAMESPACE_NODE_TYPES,
        "JsxNoCommentTextnodes" => REACTJSXNOCOMMENTTEXTNODES_NODE_TYPES,
        "NoRenderReturnValue" => REACTNORENDERRETURNVALUE_NODE_TYPES,
        "NoDanger" => REACTNODANGER_NODE_TYPES,
        "NoArrayIndexKey" => REACTNOARRAYINDEXKEY_NODE_TYPES,
        "NoUnknownProperty" => REACTNOUNKNOWNPROPERTY_NODE_TYPES,
        "NoMagicArrayFlatDepth" => UNICORNNOMAGICARRAYFLATDEPTH_NODE_TYPES,
        "PreferObjectFromEntries" => UNICORNPREFEROBJECTFROMENTRIES_NODE_TYPES,
        "PreferEventTarget" => UNICORNPREFEREVENTTARGET_NODE_TYPES,
        "NumberLiteralCase" => UNICORNNUMBERLITERALCASE_NODE_TYPES,
        "NoThisAssignment" => UNICORNNOTHISASSIGNMENT_NODE_TYPES,
        "PreferSetHas" => UNICORNPREFERSETHAS_NODE_TYPES,
        "PreferArrayFind" => UNICORNPREFERARRAYFIND_NODE_TYPES,
        "NoLonelyIf" => UNICORNNOLONELYIF_NODE_TYPES,
        "NoTypeofUndefined" => UNICORNNOTYPEOFUNDEFINED_NODE_TYPES,
        "PreferSetSize" => UNICORNPREFERSETSIZE_NODE_TYPES,
        "PreferMathMinMax" => UNICORNPREFERMATHMINMAX_NODE_TYPES,
        "PreferStringTrimStartEnd" => UNICORNPREFERSTRINGTRIMSTARTEND_NODE_TYPES,
        "PreferNodeProtocol" => UNICORNPREFERNODEPROTOCOL_NODE_TYPES,
        "NoUselessSwitchCase" => UNICORNNOUSELESSSWITCHCASE_NODE_TYPES,
        "NoAnonymousDefaultExport" => UNICORNNOANONYMOUSDEFAULTEXPORT_NODE_TYPES,
        "RequireArrayJoinSeparator" => UNICORNREQUIREARRAYJOINSEPARATOR_NODE_TYPES,
        "PreferStringReplaceAll" => UNICORNPREFERSTRINGREPLACEALL_NODE_TYPES,
        "FilenameCase" => UNICORNFILENAMECASE_NODE_TYPES,
        "NoAccessorRecursion" => UNICORNNOACCESSORRECURSION_NODE_TYPES,
        "NoNewArray" => UNICORNNONEWARRAY_NODE_TYPES,
        "PreferModernMathApis" => UNICORNPREFERMODERNMATHAPIS_NODE_TYPES,
        "NoUselessFallbackInSpread" => UNICORNNOUSELESSFALLBACKINSPREAD_NODE_TYPES,
        "ConsistentEmptyArraySpread" => UNICORNCONSISTENTEMPTYARRAYSPREAD_NODE_TYPES,
        "ConsistentExistenceIndexCheck" => UNICORNCONSISTENTEXISTENCEINDEXCHECK_NODE_TYPES,
        "EmptyBraceSpaces" => UNICORNEMPTYBRACESPACES_NODE_TYPES,
        "NoArrayMethodThisArgument" => UNICORNNOARRAYMETHODTHISARGUMENT_NODE_TYPES,
        "PreferStringRaw" => UNICORNPREFERSTRINGRAW_NODE_TYPES,
        "ConsistentDateClone" => UNICORNCONSISTENTDATECLONE_NODE_TYPES,
        "ConsistentAssert" => UNICORNCONSISTENTASSERT_NODE_TYPES,
        "NoInvalidFetchOptions" => UNICORNNOINVALIDFETCHOPTIONS_NODE_TYPES,
        "PreferSpread" => UNICORNPREFERSPREAD_NODE_TYPES,
        "PreferDomNodeAppend" => UNICORNPREFERDOMNODEAPPEND_NODE_TYPES,
        "PreferArrayIndexOf" => UNICORNPREFERARRAYINDEXOF_NODE_TYPES,
        "PreferOptionalCatchBinding" => UNICORNPREFEROPTIONALCATCHBINDING_NODE_TYPES,
        "SwitchCaseBraces" => UNICORNSWITCHCASEBRACES_NODE_TYPES,
        "PreferArraySome" => UNICORNPREFERARRAYSOME_NODE_TYPES,
        "NoNewBuffer" => UNICORNNONEWBUFFER_NODE_TYPES,
        "PreferNativeCoercionFunctions" => UNICORNPREFERNATIVECOERCIONFUNCTIONS_NODE_TYPES,
        "NoDocumentCookie" => UNICORNNODOCUMENTCOOKIE_NODE_TYPES,
        "NoNestedTernary" => UNICORNNONESTEDTERNARY_NODE_TYPES,
        "NoZeroFractions" => UNICORNNOZEROFRACTIONS_NODE_TYPES,
        "NoAwaitInPromiseMethods" => UNICORNNOAWAITINPROMISEMETHODS_NODE_TYPES,
        "TextEncodingIdentifierCase" => UNICORNTEXTENCODINGIDENTIFIERCASE_NODE_TYPES,
        "ThrowNewError" => UNICORNTHROWNEWERROR_NODE_TYPES,
        "NoUselessLengthCheck" => UNICORNNOUSELESSLENGTHCHECK_NODE_TYPES,
        "ConsistentFunctionScoping" => UNICORNCONSISTENTFUNCTIONSCOPING_NODE_TYPES,
        "PreferIncludes" => UNICORNPREFERINCLUDES_NODE_TYPES,
        "PreferPrototypeMethods" => UNICORNPREFERPROTOTYPEMETHODS_NODE_TYPES,
        "NoNegationInEqualityCheck" => UNICORNNONEGATIONINEQUALITYCHECK_NODE_TYPES,
        "ExplicitLengthCheck" => UNICORNEXPLICITLENGTHCHECK_NODE_TYPES,
        "NoArrayReduce" => UNICORNNOARRAYREDUCE_NODE_TYPES,
        "RequirePostMessageTargetOrigin" => UNICORNREQUIREPOSTMESSAGETARGETORIGIN_NODE_TYPES,
        "PreferArrayFlat" => UNICORNPREFERARRAYFLAT_NODE_TYPES,
        "CatchErrorName" => UNICORNCATCHERRORNAME_NODE_TYPES,
        "NoEmptyFile" => UNICORNNOEMPTYFILE_NODE_TYPES,
        "NoUnnecessaryAwait" => UNICORNNOUNNECESSARYAWAIT_NODE_TYPES,
        "NoUnnecessarySliceEnd" => UNICORNNOUNNECESSARYSLICEEND_NODE_TYPES,
        "PreferArrayFlatMap" => UNICORNPREFERARRAYFLATMAP_NODE_TYPES,
        "NoInvalidRemoveEventListener" => UNICORNNOINVALIDREMOVEEVENTLISTENER_NODE_TYPES,
        "NumericSeparatorsStyle" => UNICORNNUMERICSEPARATORSSTYLE_NODE_TYPES,
        "NoInstanceofBuiltins" => UNICORNNOINSTANCEOFBUILTINS_NODE_TYPES,
        "NoThenable" => UNICORNNOTHENABLE_NODE_TYPES,
        "NoProcessExit" => UNICORNNOPROCESSEXIT_NODE_TYPES,
        "NoAbusiveEslintDisable" => UNICORNNOABUSIVEESLINTDISABLE_NODE_TYPES,
        "PreferReflectApply" => UNICORNPREFERREFLECTAPPLY_NODE_TYPES,
        "PreferStructuredClone" => UNICORNPREFERSTRUCTUREDCLONE_NODE_TYPES,
        "PreferDomNodeTextContent" => UNICORNPREFERDOMNODETEXTCONTENT_NODE_TYPES,
        "NewForBuiltins" => UNICORNNEWFORBUILTINS_NODE_TYPES,
        "NoLengthAsSliceEnd" => UNICORNNOLENGTHASSLICEEND_NODE_TYPES,
        "NoHexEscape" => UNICORNNOHEXESCAPE_NODE_TYPES,
        "PreferQuerySelector" => UNICORNPREFERQUERYSELECTOR_NODE_TYPES,
        "PreferNegativeIndex" => UNICORNPREFERNEGATIVEINDEX_NODE_TYPES,
        "PreferTypeError" => UNICORNPREFERTYPEERROR_NODE_TYPES,
        "PreferBlobReadingMethods" => UNICORNPREFERBLOBREADINGMETHODS_NODE_TYPES,
        "NoNull" => UNICORNNONULL_NODE_TYPES,
        "PreferStringSlice" => UNICORNPREFERSTRINGSLICE_NODE_TYPES,
        "PreferMathTrunc" => UNICORNPREFERMATHTRUNC_NODE_TYPES,
        "NoUnreadableArrayDestructuring" => UNICORNNOUNREADABLEARRAYDESTRUCTURING_NODE_TYPES,
        "PreferCodePoint" => UNICORNPREFERCODEPOINT_NODE_TYPES,
        "NoConsoleSpaces" => UNICORNNOCONSOLESPACES_NODE_TYPES,
        "PreferStringStartsEndsWith" => UNICORNPREFERSTRINGSTARTSENDSWITH_NODE_TYPES,
        "NoSinglePromiseInPromiseMethods" => UNICORNNOSINGLEPROMISEINPROMISEMETHODS_NODE_TYPES,
        "PreferNumberProperties" => UNICORNPREFERNUMBERPROPERTIES_NODE_TYPES,
        "NoAwaitExpressionMember" => UNICORNNOAWAITEXPRESSIONMEMBER_NODE_TYPES,
        "NoInstanceofArray" => UNICORNNOINSTANCEOFARRAY_NODE_TYPES,
        "ErrorMessage" => UNICORNERRORMESSAGE_NODE_TYPES,
        "PreferDomNodeRemove" => UNICORNPREFERDOMNODEREMOVE_NODE_TYPES,
        "PreferAddEventListener" => UNICORNPREFERADDEVENTLISTENER_NODE_TYPES,
        "PreferDateNow" => UNICORNPREFERDATENOW_NODE_TYPES,
        "NoUselessUndefined" => UNICORNNOUSELESSUNDEFINED_NODE_TYPES,
        "NoStaticOnlyClass" => UNICORNNOSTATICONLYCLASS_NODE_TYPES,
        "NoUselessPromiseResolveReject" => UNICORNNOUSELESSPROMISERESOLVEREJECT_NODE_TYPES,
        "EscapeCase" => UNICORNESCAPECASE_NODE_TYPES,
        "PreferLogicalOperatorOverTernary" => UNICORNPREFERLOGICALOPERATOROVERTERNARY_NODE_TYPES,
        "NoObjectAsDefaultParameter" => UNICORNNOOBJECTASDEFAULTPARAMETER_NODE_TYPES,
        "NoUnnecessaryArrayFlatDepth" => UNICORNNOUNNECESSARYARRAYFLATDEPTH_NODE_TYPES,
        "ConstEval" => NOUSELESSSPREADCONSTEVAL_NODE_TYPES,
        "Mod" => NOUSELESSSPREADMOD_NODE_TYPES,
        "PreferDomNodeDataset" => UNICORNPREFERDOMNODEDATASET_NODE_TYPES,
        "NoUnreadableIife" => UNICORNNOUNREADABLEIIFE_NODE_TYPES,
        "PreferModernDomApis" => UNICORNPREFERMODERNDOMAPIS_NODE_TYPES,
        "NoArrayForEach" => UNICORNNOARRAYFOREACH_NODE_TYPES,
        "PreferGlobalThis" => UNICORNPREFERGLOBALTHIS_NODE_TYPES,
        "PreferRegexpTest" => UNICORNPREFERREGEXPTEST_NODE_TYPES,
        "RequireNumberToFixedDigitsArgument" => UNICORNREQUIRENUMBERTOFIXEDDIGITSARGUMENT_NODE_TYPES,
        _ => &[], // Fallback for unknown rules
    }
}

/// Get any_node_type flag for a rule by its struct name
pub fn get_any_node_type(rule_name: &str) -> bool {
    match rule_name {
        "PromiseFunctionAsync" => TYPESCRIPTPROMISEFUNCTIONASYNC_ANY_NODE_TYPE,
        "NoFloatingPromises" => TYPESCRIPTNOFLOATINGPROMISES_ANY_NODE_TYPE,
        "NoUnnecessaryParameterPropertyAssignment" => TYPESCRIPTNOUNNECESSARYPARAMETERPROPERTYASSIGNMENT_ANY_NODE_TYPE,
        "NoUnnecessaryBooleanLiteralCompare" => TYPESCRIPTNOUNNECESSARYBOOLEANLITERALCOMPARE_ANY_NODE_TYPE,
        "NonNullableTypeAssertionStyle" => TYPESCRIPTNONNULLABLETYPEASSERTIONSTYLE_ANY_NODE_TYPE,
        "NoNonNullAssertedOptionalChain" => TYPESCRIPTNONONNULLASSERTEDOPTIONALCHAIN_ANY_NODE_TYPE,
        "NoUnsafeCall" => TYPESCRIPTNOUNSAFECALL_ANY_NODE_TYPE,
        "PreferFunctionType" => TYPESCRIPTPREFERFUNCTIONTYPE_ANY_NODE_TYPE,
        "ExplicitFunctionReturnType" => TYPESCRIPTEXPLICITFUNCTIONRETURNTYPE_ANY_NODE_TYPE,
        "SwitchExhaustivenessCheck" => TYPESCRIPTSWITCHEXHAUSTIVENESSCHECK_ANY_NODE_TYPE,
        "NoNonNullAssertion" => TYPESCRIPTNONONNULLASSERTION_ANY_NODE_TYPE,
        "RequireArraySortCompare" => TYPESCRIPTREQUIREARRAYSORTCOMPARE_ANY_NODE_TYPE,
        "NoUnsafeMemberAccess" => TYPESCRIPTNOUNSAFEMEMBERACCESS_ANY_NODE_TYPE,
        "NoDuplicateTypeConstituents" => TYPESCRIPTNODUPLICATETYPECONSTITUENTS_ANY_NODE_TYPE,
        "PreferEnumInitializers" => TYPESCRIPTPREFERENUMINITIALIZERS_ANY_NODE_TYPE,
        "NoArrayDelete" => TYPESCRIPTNOARRAYDELETE_ANY_NODE_TYPE,
        "NoUselessEmptyExport" => TYPESCRIPTNOUSELESSEMPTYEXPORT_ANY_NODE_TYPE,
        "ReturnAwait" => TYPESCRIPTRETURNAWAIT_ANY_NODE_TYPE,
        "NoMisusedSpread" => TYPESCRIPTNOMISUSEDSPREAD_ANY_NODE_TYPE,
        "RelatedGetterSetterPairs" => TYPESCRIPTRELATEDGETTERSETTERPAIRS_ANY_NODE_TYPE,
        "ArrayType" => TYPESCRIPTARRAYTYPE_ANY_NODE_TYPE,
        "PreferReduceTypeParameter" => TYPESCRIPTPREFERREDUCETYPEPARAMETER_ANY_NODE_TYPE,
        "PreferLiteralEnumMember" => TYPESCRIPTPREFERLITERALENUMMEMBER_ANY_NODE_TYPE,
        "UseUnknownInCatchCallbackVariable" => TYPESCRIPTUSEUNKNOWNINCATCHCALLBACKVARIABLE_ANY_NODE_TYPE,
        "NoDuplicateEnumValues" => TYPESCRIPTNODUPLICATEENUMVALUES_ANY_NODE_TYPE,
        "NoMisusedPromises" => TYPESCRIPTNOMISUSEDPROMISES_ANY_NODE_TYPE,
        "PreferForOf" => TYPESCRIPTPREFERFOROF_ANY_NODE_TYPE,
        "ConsistentIndexedObjectStyle" => TYPESCRIPTCONSISTENTINDEXEDOBJECTSTYLE_ANY_NODE_TYPE,
        "NoUnsafeAssignment" => TYPESCRIPTNOUNSAFEASSIGNMENT_ANY_NODE_TYPE,
        "NoRequireImports" => TYPESCRIPTNOREQUIREIMPORTS_ANY_NODE_TYPE,
        "NoRedundantTypeConstituents" => TYPESCRIPTNOREDUNDANTTYPECONSTITUENTS_ANY_NODE_TYPE,
        "PreferNamespaceKeyword" => TYPESCRIPTPREFERNAMESPACEKEYWORD_ANY_NODE_TYPE,
        "NoConfusingNonNullAssertion" => TYPESCRIPTNOCONFUSINGNONNULLASSERTION_ANY_NODE_TYPE,
        "NoThisAlias" => TYPESCRIPTNOTHISALIAS_ANY_NODE_TYPE,
        "NoUnsafeEnumComparison" => TYPESCRIPTNOUNSAFEENUMCOMPARISON_ANY_NODE_TYPE,
        "NoVarRequires" => TYPESCRIPTNOVARREQUIRES_ANY_NODE_TYPE,
        "AdjacentOverloadSignatures" => TYPESCRIPTADJACENTOVERLOADSIGNATURES_ANY_NODE_TYPE,
        "UnboundMethod" => TYPESCRIPTUNBOUNDMETHOD_ANY_NODE_TYPE,
        "NoWrapperObjectTypes" => TYPESCRIPTNOWRAPPEROBJECTTYPES_ANY_NODE_TYPE,
        "NoExplicitAny" => TYPESCRIPTNOEXPLICITANY_ANY_NODE_TYPE,
        "NoImportTypeSideEffects" => TYPESCRIPTNOIMPORTTYPESIDEEFFECTS_ANY_NODE_TYPE,
        "NoUnsafeFunctionType" => TYPESCRIPTNOUNSAFEFUNCTIONTYPE_ANY_NODE_TYPE,
        "NoConfusingVoidExpression" => TYPESCRIPTNOCONFUSINGVOIDEXPRESSION_ANY_NODE_TYPE,
        "NoUnsafeDeclarationMerging" => TYPESCRIPTNOUNSAFEDECLARATIONMERGING_ANY_NODE_TYPE,
        "NoMixedEnums" => TYPESCRIPTNOMIXEDENUMS_ANY_NODE_TYPE,
        "NoUnnecessaryTypeAssertion" => TYPESCRIPTNOUNNECESSARYTYPEASSERTION_ANY_NODE_TYPE,
        "NoBaseToString" => TYPESCRIPTNOBASETOSTRING_ANY_NODE_TYPE,
        "ConsistentTypeImports" => TYPESCRIPTCONSISTENTTYPEIMPORTS_ANY_NODE_TYPE,
        "PreferTsExpectError" => TYPESCRIPTPREFERTSEXPECTERROR_ANY_NODE_TYPE,
        "RestrictPlusOperands" => TYPESCRIPTRESTRICTPLUSOPERANDS_ANY_NODE_TYPE,
        "NoImpliedEval" => TYPESCRIPTNOIMPLIEDEVAL_ANY_NODE_TYPE,
        "OnlyThrowError" => TYPESCRIPTONLYTHROWERROR_ANY_NODE_TYPE,
        "AwaitThenable" => TYPESCRIPTAWAITTHENABLE_ANY_NODE_TYPE,
        "PreferReturnThisType" => TYPESCRIPTPREFERRETURNTHISTYPE_ANY_NODE_TYPE,
        "PreferAsConst" => TYPESCRIPTPREFERASCONST_ANY_NODE_TYPE,
        "PreferPromiseRejectErrors" => TYPESCRIPTPREFERPROMISEREJECTERRORS_ANY_NODE_TYPE,
        "NoInferrableTypes" => TYPESCRIPTNOINFERRABLETYPES_ANY_NODE_TYPE,
        "ExplicitModuleBoundaryTypes" => TYPESCRIPTEXPLICITMODULEBOUNDARYTYPES_ANY_NODE_TYPE,
        "NoMisusedNew" => TYPESCRIPTNOMISUSEDNEW_ANY_NODE_TYPE,
        "NoDynamicDelete" => TYPESCRIPTNODYNAMICDELETE_ANY_NODE_TYPE,
        "NoNonNullAssertedNullishCoalescing" => TYPESCRIPTNONONNULLASSERTEDNULLISHCOALESCING_ANY_NODE_TYPE,
        "BanTypes" => TYPESCRIPTBANTYPES_ANY_NODE_TYPE,
        "NoUnsafeReturn" => TYPESCRIPTNOUNSAFERETURN_ANY_NODE_TYPE,
        "NoMeaninglessVoidOperator" => TYPESCRIPTNOMEANINGLESSVOIDOPERATOR_ANY_NODE_TYPE,
        "NoUnnecessaryTemplateExpression" => TYPESCRIPTNOUNNECESSARYTEMPLATEEXPRESSION_ANY_NODE_TYPE,
        "NoUnsafeUnaryMinus" => TYPESCRIPTNOUNSAFEUNARYMINUS_ANY_NODE_TYPE,
        "NoEmptyObjectType" => TYPESCRIPTNOEMPTYOBJECTTYPE_ANY_NODE_TYPE,
        "ConsistentTypeDefinitions" => TYPESCRIPTCONSISTENTTYPEDEFINITIONS_ANY_NODE_TYPE,
        "NoExtraneousClass" => TYPESCRIPTNOEXTRANEOUSCLASS_ANY_NODE_TYPE,
        "ConsistentGenericConstructors" => TYPESCRIPTCONSISTENTGENERICCONSTRUCTORS_ANY_NODE_TYPE,
        "NoNamespace" => TYPESCRIPTNONAMESPACE_ANY_NODE_TYPE,
        "RequireAwait" => TYPESCRIPTREQUIREAWAIT_ANY_NODE_TYPE,
        "RestrictTemplateExpressions" => TYPESCRIPTRESTRICTTEMPLATEEXPRESSIONS_ANY_NODE_TYPE,
        "BanTslintComment" => TYPESCRIPTBANTSLINTCOMMENT_ANY_NODE_TYPE,
        "NoExtraNonNullAssertion" => TYPESCRIPTNOEXTRANONNULLASSERTION_ANY_NODE_TYPE,
        "NoUnnecessaryTypeConstraint" => TYPESCRIPTNOUNNECESSARYTYPECONSTRAINT_ANY_NODE_TYPE,
        "NoUnnecessaryTypeArguments" => TYPESCRIPTNOUNNECESSARYTYPEARGUMENTS_ANY_NODE_TYPE,
        "NoUnsafeArgument" => TYPESCRIPTNOUNSAFEARGUMENT_ANY_NODE_TYPE,
        "NoForInArray" => TYPESCRIPTNOFORINARRAY_ANY_NODE_TYPE,
        "BanTsComment" => TYPESCRIPTBANTSCOMMENT_ANY_NODE_TYPE,
        "NoUnsafeTypeAssertion" => TYPESCRIPTNOUNSAFETYPEASSERTION_ANY_NODE_TYPE,
        "TripleSlashReference" => TYPESCRIPTTRIPLESLASHREFERENCE_ANY_NODE_TYPE,
        "NoEmptyInterface" => TYPESCRIPTNOEMPTYINTERFACE_ANY_NODE_TYPE,
        "RequireLocalTestContextForConcurrentSnapshots" => VITESTREQUIRELOCALTESTCONTEXTFORCONCURRENTSNAPSHOTS_ANY_NODE_TYPE,
        "PreferToBeTruthy" => VITESTPREFERTOBETRUTHY_ANY_NODE_TYPE,
        "NoConditionalTests" => VITESTNOCONDITIONALTESTS_ANY_NODE_TYPE,
        "NoImportNodeTest" => VITESTNOIMPORTNODETEST_ANY_NODE_TYPE,
        "PreferToBeFalsy" => VITESTPREFERTOBEFALSY_ANY_NODE_TYPE,
        "PreferToBeObject" => VITESTPREFERTOBEOBJECT_ANY_NODE_TYPE,
        "NoReturnInFinally" => PROMISENORETURNINFINALLY_ANY_NODE_TYPE,
        "PreferCatch" => PROMISEPREFERCATCH_ANY_NODE_TYPE,
        "NoNesting" => PROMISENONESTING_ANY_NODE_TYPE,
        "NoNewStatics" => PROMISENONEWSTATICS_ANY_NODE_TYPE,
        "SpecOnly" => PROMISESPECONLY_ANY_NODE_TYPE,
        "NoCallbackInPromise" => PROMISENOCALLBACKINPROMISE_ANY_NODE_TYPE,
        "AvoidNew" => PROMISEAVOIDNEW_ANY_NODE_TYPE,
        "NoReturnWrap" => PROMISENORETURNWRAP_ANY_NODE_TYPE,
        "NoPromiseInCallback" => PROMISENOPROMISEINCALLBACK_ANY_NODE_TYPE,
        "ValidParams" => PROMISEVALIDPARAMS_ANY_NODE_TYPE,
        "PreferAwaitToCallbacks" => PROMISEPREFERAWAITTOCALLBACKS_ANY_NODE_TYPE,
        "ParamNames" => PROMISEPARAMNAMES_ANY_NODE_TYPE,
        "CatchOrReturn" => PROMISECATCHORRETURN_ANY_NODE_TYPE,
        "PreferAwaitToThen" => PROMISEPREFERAWAITTOTHEN_ANY_NODE_TYPE,
        "PreferHooksInOrder" => JESTPREFERHOOKSINORDER_ANY_NODE_TYPE,
        "NoHooks" => JESTNOHOOKS_ANY_NODE_TYPE,
        "PreferEqualityMatcher" => JESTPREFEREQUALITYMATCHER_ANY_NODE_TYPE,
        "ValidExpect" => JESTVALIDEXPECT_ANY_NODE_TYPE,
        "NoConditionalInTest" => JESTNOCONDITIONALINTEST_ANY_NODE_TYPE,
        "ExpectExpect" => JESTEXPECTEXPECT_ANY_NODE_TYPE,
        "PreferCalledWith" => JESTPREFERCALLEDWITH_ANY_NODE_TYPE,
        "NoLargeSnapshots" => JESTNOLARGESNAPSHOTS_ANY_NODE_TYPE,
        "NoTestReturnStatement" => JESTNOTESTRETURNSTATEMENT_ANY_NODE_TYPE,
        "NoMocksImport" => JESTNOMOCKSIMPORT_ANY_NODE_TYPE,
        "NoRestrictedMatchers" => JESTNORESTRICTEDMATCHERS_ANY_NODE_TYPE,
        "NoDuplicateHooks" => JESTNODUPLICATEHOOKS_ANY_NODE_TYPE,
        "NoCommentedOutTests" => JESTNOCOMMENTEDOUTTESTS_ANY_NODE_TYPE,
        "PreferStrictEqual" => JESTPREFERSTRICTEQUAL_ANY_NODE_TYPE,
        "PreferComparisonMatcher" => JESTPREFERCOMPARISONMATCHER_ANY_NODE_TYPE,
        "NoInterpolationInSnapshots" => JESTNOINTERPOLATIONINSNAPSHOTS_ANY_NODE_TYPE,
        "PreferToContain" => JESTPREFERTOCONTAIN_ANY_NODE_TYPE,
        "PreferTodo" => JESTPREFERTODO_ANY_NODE_TYPE,
        "Mod" => NOSTANDALONEEXPECTMOD_ANY_NODE_TYPE,
        "NoDisabledTests" => JESTNODISABLEDTESTS_ANY_NODE_TYPE,
        "NoJasmineGlobals" => JESTNOJASMINEGLOBALS_ANY_NODE_TYPE,
        "Mod" => PREFERLOWERCASETITLEMOD_ANY_NODE_TYPE,
        "PreferToHaveLength" => JESTPREFERTOHAVELENGTH_ANY_NODE_TYPE,
        "ValidTitle" => JESTVALIDTITLE_ANY_NODE_TYPE,
        "PreferHooksOnTop" => JESTPREFERHOOKSONTOP_ANY_NODE_TYPE,
        "PreferExpectResolves" => JESTPREFEREXPECTRESOLVES_ANY_NODE_TYPE,
        "PreferJestMocked" => JESTPREFERJESTMOCKED_ANY_NODE_TYPE,
        "ValidDescribeCallback" => JESTVALIDDESCRIBECALLBACK_ANY_NODE_TYPE,
        "PreferMockPromiseShorthand" => JESTPREFERMOCKPROMISESHORTHAND_ANY_NODE_TYPE,
        "NoConfusingSetTimeout" => JESTNOCONFUSINGSETTIMEOUT_ANY_NODE_TYPE,
        "NoTestPrefixes" => JESTNOTESTPREFIXES_ANY_NODE_TYPE,
        "NoConditionalExpect" => JESTNOCONDITIONALEXPECT_ANY_NODE_TYPE,
        "NoFocusedTests" => JESTNOFOCUSEDTESTS_ANY_NODE_TYPE,
        "PreferEach" => JESTPREFEREACH_ANY_NODE_TYPE,
        "NoIdenticalTitle" => JESTNOIDENTICALTITLE_ANY_NODE_TYPE,
        "NoRestrictedJestMethods" => JESTNORESTRICTEDJESTMETHODS_ANY_NODE_TYPE,
        "PreferToBe" => JESTPREFERTOBE_ANY_NODE_TYPE,
        "MaxExpects" => JESTMAXEXPECTS_ANY_NODE_TYPE,
        "NoDeprecatedFunctions" => JESTNODEPRECATEDFUNCTIONS_ANY_NODE_TYPE,
        "RequireHook" => JESTREQUIREHOOK_ANY_NODE_TYPE,
        "MaxNestedDescribe" => JESTMAXNESTEDDESCRIBE_ANY_NODE_TYPE,
        "RequireToThrowMessage" => JESTREQUIRETOTHROWMESSAGE_ANY_NODE_TYPE,
        "ConsistentTestIt" => JESTCONSISTENTTESTIT_ANY_NODE_TYPE,
        "NoDoneCallback" => JESTNODONECALLBACK_ANY_NODE_TYPE,
        "RequireTopLevelDescribe" => JESTREQUIRETOPLEVELDESCRIBE_ANY_NODE_TYPE,
        "NoAliasMethods" => JESTNOALIASMETHODS_ANY_NODE_TYPE,
        "NoExport" => JESTNOEXPORT_ANY_NODE_TYPE,
        "PreferSpyOn" => JESTPREFERSPYON_ANY_NODE_TYPE,
        "NoUntypedMockFactory" => JESTNOUNTYPEDMOCKFACTORY_ANY_NODE_TYPE,
        "Lang" => JSXA11YLANG_ANY_NODE_TYPE,
        "PreferTagOverRole" => JSXA11YPREFERTAGOVERROLE_ANY_NODE_TYPE,
        "AriaActivedescendantHasTabindex" => JSXA11YARIAACTIVEDESCENDANTHASTABINDEX_ANY_NODE_TYPE,
        "NoRedundantRoles" => JSXA11YNOREDUNDANTROLES_ANY_NODE_TYPE,
        "ImgRedundantAlt" => JSXA11YIMGREDUNDANTALT_ANY_NODE_TYPE,
        "AriaProps" => JSXA11YARIAPROPS_ANY_NODE_TYPE,
        "AriaUnsupportedElements" => JSXA11YARIAUNSUPPORTEDELEMENTS_ANY_NODE_TYPE,
        "RoleHasRequiredAriaProps" => JSXA11YROLEHASREQUIREDARIAPROPS_ANY_NODE_TYPE,
        "ClickEventsHaveKeyEvents" => JSXA11YCLICKEVENTSHAVEKEYEVENTS_ANY_NODE_TYPE,
        "TabindexNoPositive" => JSXA11YTABINDEXNOPOSITIVE_ANY_NODE_TYPE,
        "NoAutofocus" => JSXA11YNOAUTOFOCUS_ANY_NODE_TYPE,
        "LabelHasAssociatedControl" => JSXA11YLABELHASASSOCIATEDCONTROL_ANY_NODE_TYPE,
        "MediaHasCaption" => JSXA11YMEDIAHASCAPTION_ANY_NODE_TYPE,
        "AutocompleteValid" => JSXA11YAUTOCOMPLETEVALID_ANY_NODE_TYPE,
        "Scope" => JSXA11YSCOPE_ANY_NODE_TYPE,
        "AriaRole" => JSXA11YARIAROLE_ANY_NODE_TYPE,
        "NoDistractingElements" => JSXA11YNODISTRACTINGELEMENTS_ANY_NODE_TYPE,
        "HeadingHasContent" => JSXA11YHEADINGHASCONTENT_ANY_NODE_TYPE,
        "HtmlHasLang" => JSXA11YHTMLHASLANG_ANY_NODE_TYPE,
        "IframeHasTitle" => JSXA11YIFRAMEHASTITLE_ANY_NODE_TYPE,
        "NoNoninteractiveTabindex" => JSXA11YNONONINTERACTIVETABINDEX_ANY_NODE_TYPE,
        "AnchorAmbiguousText" => JSXA11YANCHORAMBIGUOUSTEXT_ANY_NODE_TYPE,
        "AnchorHasContent" => JSXA11YANCHORHASCONTENT_ANY_NODE_TYPE,
        "AnchorIsValid" => JSXA11YANCHORISVALID_ANY_NODE_TYPE,
        "AltText" => JSXA11YALTTEXT_ANY_NODE_TYPE,
        "NoAccessKey" => JSXA11YNOACCESSKEY_ANY_NODE_TYPE,
        "MouseEventsHaveKeyEvents" => JSXA11YMOUSEEVENTSHAVEKEYEVENTS_ANY_NODE_TYPE,
        "RoleSupportsAriaProps" => JSXA11YROLESUPPORTSARIAPROPS_ANY_NODE_TYPE,
        "NoAriaHiddenOnFocusable" => JSXA11YNOARIAHIDDENONFOCUSABLE_ANY_NODE_TYPE,
        "NoNewRequire" => NODENONEWREQUIRE_ANY_NODE_TYPE,
        "NoExportsAssign" => NODENOEXPORTSASSIGN_ANY_NODE_TYPE,
        "GoogleFontDisplay" => NEXTJSGOOGLEFONTDISPLAY_ANY_NODE_TYPE,
        "NoUnwantedPolyfillio" => NEXTJSNOUNWANTEDPOLYFILLIO_ANY_NODE_TYPE,
        "NoCssTags" => NEXTJSNOCSSTAGS_ANY_NODE_TYPE,
        "NoHtmlLinkForPages" => NEXTJSNOHTMLLINKFORPAGES_ANY_NODE_TYPE,
        "NoStyledJsxInDocument" => NEXTJSNOSTYLEDJSXINDOCUMENT_ANY_NODE_TYPE,
        "NoDuplicateHead" => NEXTJSNODUPLICATEHEAD_ANY_NODE_TYPE,
        "NoPageCustomFont" => NEXTJSNOPAGECUSTOMFONT_ANY_NODE_TYPE,
        "NoHeadImportInDocument" => NEXTJSNOHEADIMPORTINDOCUMENT_ANY_NODE_TYPE,
        "NoBeforeInteractiveScriptOutsideDocument" => NEXTJSNOBEFOREINTERACTIVESCRIPTOUTSIDEDOCUMENT_ANY_NODE_TYPE,
        "GoogleFontPreconnect" => NEXTJSGOOGLEFONTPRECONNECT_ANY_NODE_TYPE,
        "NoSyncScripts" => NEXTJSNOSYNCSCRIPTS_ANY_NODE_TYPE,
        "NoHeadElement" => NEXTJSNOHEADELEMENT_ANY_NODE_TYPE,
        "NoImgElement" => NEXTJSNOIMGELEMENT_ANY_NODE_TYPE,
        "InlineScriptId" => NEXTJSINLINESCRIPTID_ANY_NODE_TYPE,
        "NextScriptForGa" => NEXTJSNEXTSCRIPTFORGA_ANY_NODE_TYPE,
        "NoScriptComponentInHead" => NEXTJSNOSCRIPTCOMPONENTINHEAD_ANY_NODE_TYPE,
        "NoTitleInDocumentHead" => NEXTJSNOTITLEINDOCUMENTHEAD_ANY_NODE_TYPE,
        "NoDocumentImportInPage" => NEXTJSNODOCUMENTIMPORTINPAGE_ANY_NODE_TYPE,
        "NoAssignModuleVariable" => NEXTJSNOASSIGNMODULEVARIABLE_ANY_NODE_TYPE,
        "NoTypos" => NEXTJSNOTYPOS_ANY_NODE_TYPE,
        "NoAsyncClientComponent" => NEXTJSNOASYNCCLIENTCOMPONENT_ANY_NODE_TYPE,
        "ExportsLast" => IMPORTEXPORTSLAST_ANY_NODE_TYPE,
        "Export" => IMPORTEXPORT_ANY_NODE_TYPE,
        "NoNamedDefault" => IMPORTNONAMEDDEFAULT_ANY_NODE_TYPE,
        "NoAnonymousDefaultExport" => IMPORTNOANONYMOUSDEFAULTEXPORT_ANY_NODE_TYPE,
        "NoWebpackLoaderSyntax" => IMPORTNOWEBPACKLOADERSYNTAX_ANY_NODE_TYPE,
        "Named" => IMPORTNAMED_ANY_NODE_TYPE,
        "First" => IMPORTFIRST_ANY_NODE_TYPE,
        "NoDuplicates" => IMPORTNODUPLICATES_ANY_NODE_TYPE,
        "NoDefaultExport" => IMPORTNODEFAULTEXPORT_ANY_NODE_TYPE,
        "ConsistentTypeSpecifierStyle" => IMPORTCONSISTENTTYPESPECIFIERSTYLE_ANY_NODE_TYPE,
        "GroupExports" => IMPORTGROUPEXPORTS_ANY_NODE_TYPE,
        "NoNamedAsDefaultMember" => IMPORTNONAMEDASDEFAULTMEMBER_ANY_NODE_TYPE,
        "Namespace" => IMPORTNAMESPACE_ANY_NODE_TYPE,
        "NoAbsolutePath" => IMPORTNOABSOLUTEPATH_ANY_NODE_TYPE,
        "Extensions" => IMPORTEXTENSIONS_ANY_NODE_TYPE,
        "PreferDefaultExport" => IMPORTPREFERDEFAULTEXPORT_ANY_NODE_TYPE,
        "NoMutableExports" => IMPORTNOMUTABLEEXPORTS_ANY_NODE_TYPE,
        "NoCycle" => IMPORTNOCYCLE_ANY_NODE_TYPE,
        "NoCommonjs" => IMPORTNOCOMMONJS_ANY_NODE_TYPE,
        "NoDynamicRequire" => IMPORTNODYNAMICREQUIRE_ANY_NODE_TYPE,
        "NoNamespace" => IMPORTNONAMESPACE_ANY_NODE_TYPE,
        "NoAmd" => IMPORTNOAMD_ANY_NODE_TYPE,
        "NoEmptyNamedBlocks" => IMPORTNOEMPTYNAMEDBLOCKS_ANY_NODE_TYPE,
        "NoSelfImport" => IMPORTNOSELFIMPORT_ANY_NODE_TYPE,
        "NoNamedAsDefault" => IMPORTNONAMEDASDEFAULT_ANY_NODE_TYPE,
        "Unambiguous" => IMPORTUNAMBIGUOUS_ANY_NODE_TYPE,
        "NoUnassignedImport" => IMPORTNOUNASSIGNEDIMPORT_ANY_NODE_TYPE,
        "MaxDependencies" => IMPORTMAXDEPENDENCIES_ANY_NODE_TYPE,
        "Default" => IMPORTDEFAULT_ANY_NODE_TYPE,
        "EmptyTags" => JSDOCEMPTYTAGS_ANY_NODE_TYPE,
        "ImplementsOnClasses" => JSDOCIMPLEMENTSONCLASSES_ANY_NODE_TYPE,
        "RequirePropertyType" => JSDOCREQUIREPROPERTYTYPE_ANY_NODE_TYPE,
        "CheckAccess" => JSDOCCHECKACCESS_ANY_NODE_TYPE,
        "RequireReturns" => JSDOCREQUIRERETURNS_ANY_NODE_TYPE,
        "RequireProperty" => JSDOCREQUIREPROPERTY_ANY_NODE_TYPE,
        "RequireReturnsDescription" => JSDOCREQUIRERETURNSDESCRIPTION_ANY_NODE_TYPE,
        "CheckPropertyNames" => JSDOCCHECKPROPERTYNAMES_ANY_NODE_TYPE,
        "RequireReturnsType" => JSDOCREQUIRERETURNSTYPE_ANY_NODE_TYPE,
        "RequireYields" => JSDOCREQUIREYIELDS_ANY_NODE_TYPE,
        "RequireParamDescription" => JSDOCREQUIREPARAMDESCRIPTION_ANY_NODE_TYPE,
        "RequireParamName" => JSDOCREQUIREPARAMNAME_ANY_NODE_TYPE,
        "NoDefaults" => JSDOCNODEFAULTS_ANY_NODE_TYPE,
        "RequirePropertyDescription" => JSDOCREQUIREPROPERTYDESCRIPTION_ANY_NODE_TYPE,
        "RequireParam" => JSDOCREQUIREPARAM_ANY_NODE_TYPE,
        "RequirePropertyName" => JSDOCREQUIREPROPERTYNAME_ANY_NODE_TYPE,
        "CheckTagNames" => JSDOCCHECKTAGNAMES_ANY_NODE_TYPE,
        "RequireParamType" => JSDOCREQUIREPARAMTYPE_ANY_NODE_TYPE,
        "NoRestrictedGlobals" => ESLINTNORESTRICTEDGLOBALS_ANY_NODE_TYPE,
        "NoNewFunc" => ESLINTNONEWFUNC_ANY_NODE_TYPE,
        "RequireYield" => ESLINTREQUIREYIELD_ANY_NODE_TYPE,
        "NoUndef" => ESLINTNOUNDEF_ANY_NODE_TYPE,
        "GetterReturn" => ESLINTGETTERRETURN_ANY_NODE_TYPE,
        "Radix" => ESLINTRADIX_ANY_NODE_TYPE,
        "NoGlobalAssign" => ESLINTNOGLOBALASSIGN_ANY_NODE_TYPE,
        "NoUnusedExpressions" => ESLINTNOUNUSEDEXPRESSIONS_ANY_NODE_TYPE,
        "IdLength" => ESLINTIDLENGTH_ANY_NODE_TYPE,
        "NoSetterReturn" => ESLINTNOSETTERRETURN_ANY_NODE_TYPE,
        "NoConstantCondition" => ESLINTNOCONSTANTCONDITION_ANY_NODE_TYPE,
        "Usage" => NOUNUSEDVARSUSAGE_ANY_NODE_TYPE,
        "BindingPattern" => NOUNUSEDVARSBINDINGPATTERN_ANY_NODE_TYPE,
        "Diagnostic" => NOUNUSEDVARSDIAGNOSTIC_ANY_NODE_TYPE,
        "Symbol" => NOUNUSEDVARSSYMBOL_ANY_NODE_TYPE,
        "Options" => NOUNUSEDVARSOPTIONS_ANY_NODE_TYPE,
        "Allowed" => NOUNUSEDVARSALLOWED_ANY_NODE_TYPE,
        "Ignored" => NOUNUSEDVARSIGNORED_ANY_NODE_TYPE,
        "Mod" => NOUNUSEDVARSMOD_ANY_NODE_TYPE,
        "NoDupeClassMembers" => ESLINTNODUPECLASSMEMBERS_ANY_NODE_TYPE,
        "PreferObjectSpread" => ESLINTPREFEROBJECTSPREAD_ANY_NODE_TYPE,
        "Yoda" => ESLINTYODA_ANY_NODE_TYPE,
        "FuncNames" => ESLINTFUNCNAMES_ANY_NODE_TYPE,
        "NoLonelyIf" => ESLINTNOLONELYIF_ANY_NODE_TYPE,
        "NoPlusplus" => ESLINTNOPLUSPLUS_ANY_NODE_TYPE,
        "NoIrregularWhitespace" => ESLINTNOIRREGULARWHITESPACE_ANY_NODE_TYPE,
        "NoArrayConstructor" => ESLINTNOARRAYCONSTRUCTOR_ANY_NODE_TYPE,
        "ValidTypeof" => ESLINTVALIDTYPEOF_ANY_NODE_TYPE,
        "NoWith" => ESLINTNOWITH_ANY_NODE_TYPE,
        "NoSelfCompare" => ESLINTNOSELFCOMPARE_ANY_NODE_TYPE,
        "MaxLinesPerFunction" => ESLINTMAXLINESPERFUNCTION_ANY_NODE_TYPE,
        "DefaultParamLast" => ESLINTDEFAULTPARAMLAST_ANY_NODE_TYPE,
        "UseIsnan" => ESLINTUSEISNAN_ANY_NODE_TYPE,
        "NoObjectConstructor" => ESLINTNOOBJECTCONSTRUCTOR_ANY_NODE_TYPE,
        "SortVars" => ESLINTSORTVARS_ANY_NODE_TYPE,
        "NoUnusedPrivateClassMembers" => ESLINTNOUNUSEDPRIVATECLASSMEMBERS_ANY_NODE_TYPE,
        "NoThrowLiteral" => ESLINTNOTHROWLITERAL_ANY_NODE_TYPE,
        "NoMultiStr" => ESLINTNOMULTISTR_ANY_NODE_TYPE,
        "NoEmptyPattern" => ESLINTNOEMPTYPATTERN_ANY_NODE_TYPE,
        "NoDeleteVar" => ESLINTNODELETEVAR_ANY_NODE_TYPE,
        "NoLossOfPrecision" => ESLINTNOLOSSOFPRECISION_ANY_NODE_TYPE,
        "SortImports" => ESLINTSORTIMPORTS_ANY_NODE_TYPE,
        "NoLabels" => ESLINTNOLABELS_ANY_NODE_TYPE,
        "PreferSpread" => ESLINTPREFERSPREAD_ANY_NODE_TYPE,
        "NoSparseArrays" => ESLINTNOSPARSEARRAYS_ANY_NODE_TYPE,
        "MaxParams" => ESLINTMAXPARAMS_ANY_NODE_TYPE,
        "NoEmptyStaticBlock" => ESLINTNOEMPTYSTATICBLOCK_ANY_NODE_TYPE,
        "NoVar" => ESLINTNOVAR_ANY_NODE_TYPE,
        "Curly" => ESLINTCURLY_ANY_NODE_TYPE,
        "NoScriptUrl" => ESLINTNOSCRIPTURL_ANY_NODE_TYPE,
        "SymbolDescription" => ESLINTSYMBOLDESCRIPTION_ANY_NODE_TYPE,
        "NoNestedTernary" => ESLINTNONESTEDTERNARY_ANY_NODE_TYPE,
        "NoUselessConcat" => ESLINTNOUSELESSCONCAT_ANY_NODE_TYPE,
        "NoIterator" => ESLINTNOITERATOR_ANY_NODE_TYPE,
        "NoDebugger" => ESLINTNODEBUGGER_ANY_NODE_TYPE,
        "NoImportAssign" => ESLINTNOIMPORTASSIGN_ANY_NODE_TYPE,
        "NoUnsafeFinally" => ESLINTNOUNSAFEFINALLY_ANY_NODE_TYPE,
        "NoTemplateCurlyInString" => ESLINTNOTEMPLATECURLYINSTRING_ANY_NODE_TYPE,
        "NoFuncAssign" => ESLINTNOFUNCASSIGN_ANY_NODE_TYPE,
        "NoEmptyCharacterClass" => ESLINTNOEMPTYCHARACTERCLASS_ANY_NODE_TYPE,
        "NoExtraBind" => ESLINTNOEXTRABIND_ANY_NODE_TYPE,
        "SortKeys" => ESLINTSORTKEYS_ANY_NODE_TYPE,
        "NoRegexSpaces" => ESLINTNOREGEXSPACES_ANY_NODE_TYPE,
        "NoClassAssign" => ESLINTNOCLASSASSIGN_ANY_NODE_TYPE,
        "NoEval" => ESLINTNOEVAL_ANY_NODE_TYPE,
        "Eqeqeq" => ESLINTEQEQEQ_ANY_NODE_TYPE,
        "NoNegatedCondition" => ESLINTNONEGATEDCONDITION_ANY_NODE_TYPE,
        "NoUnusedLabels" => ESLINTNOUNUSEDLABELS_ANY_NODE_TYPE,
        "PreferNumericLiterals" => ESLINTPREFERNUMERICLITERALS_ANY_NODE_TYPE,
        "NoUselessConstructor" => ESLINTNOUSELESSCONSTRUCTOR_ANY_NODE_TYPE,
        "BlockScopedVar" => ESLINTBLOCKSCOPEDVAR_ANY_NODE_TYPE,
        "NoBitwise" => ESLINTNOBITWISE_ANY_NODE_TYPE,
        "NoEqNull" => ESLINTNOEQNULL_ANY_NODE_TYPE,
        "NoCaseDeclarations" => ESLINTNOCASEDECLARATIONS_ANY_NODE_TYPE,
        "NoNonoctalDecimalEscape" => ESLINTNONONOCTALDECIMALESCAPE_ANY_NODE_TYPE,
        "NewCap" => ESLINTNEWCAP_ANY_NODE_TYPE,
        "NoAwaitInLoop" => ESLINTNOAWAITINLOOP_ANY_NODE_TYPE,
        "NoUnassignedVars" => ESLINTNOUNASSIGNEDVARS_ANY_NODE_TYPE,
        "NoEmpty" => ESLINTNOEMPTY_ANY_NODE_TYPE,
        "NoControlRegex" => ESLINTNOCONTROLREGEX_ANY_NODE_TYPE,
        "DefaultCase" => ESLINTDEFAULTCASE_ANY_NODE_TYPE,
        "NoDuplicateCase" => ESLINTNODUPLICATECASE_ANY_NODE_TYPE,
        "NoElseReturn" => ESLINTNOELSERETURN_ANY_NODE_TYPE,
        "NoRedeclare" => ESLINTNOREDECLARE_ANY_NODE_TYPE,
        "OperatorAssignment" => ESLINTOPERATORASSIGNMENT_ANY_NODE_TYPE,
        "NoMagicNumbers" => ESLINTNOMAGICNUMBERS_ANY_NODE_TYPE,
        "NoUselessRename" => ESLINTNOUSELESSRENAME_ANY_NODE_TYPE,
        "NoExtendNative" => ESLINTNOEXTENDNATIVE_ANY_NODE_TYPE,
        "ForDirection" => ESLINTFORDIRECTION_ANY_NODE_TYPE,
        "NoInnerDeclarations" => ESLINTNOINNERDECLARATIONS_ANY_NODE_TYPE,
        "NoMultiAssign" => ESLINTNOMULTIASSIGN_ANY_NODE_TYPE,
        "NoRestrictedImports" => ESLINTNORESTRICTEDIMPORTS_ANY_NODE_TYPE,
        "NoConstAssign" => ESLINTNOCONSTASSIGN_ANY_NODE_TYPE,
        "GroupedAccessorPairs" => ESLINTGROUPEDACCESSORPAIRS_ANY_NODE_TYPE,
        "NoDuplicateImports" => ESLINTNODUPLICATEIMPORTS_ANY_NODE_TYPE,
        "NoVoid" => ESLINTNOVOID_ANY_NODE_TYPE,
        "NoUnneededTernary" => ESLINTNOUNNEEDEDTERNARY_ANY_NODE_TYPE,
        "NoCompareNegZero" => ESLINTNOCOMPARENEGZERO_ANY_NODE_TYPE,
        "NoNew" => ESLINTNONEW_ANY_NODE_TYPE,
        "PreferPromiseRejectErrors" => ESLINTPREFERPROMISEREJECTERRORS_ANY_NODE_TYPE,
        "InitDeclarations" => ESLINTINITDECLARATIONS_ANY_NODE_TYPE,
        "PreferRestParams" => ESLINTPREFERRESTPARAMS_ANY_NODE_TYPE,
        "NoExtraLabel" => ESLINTNOEXTRALABEL_ANY_NODE_TYPE,
        "PreferExponentiationOperator" => ESLINTPREFEREXPONENTIATIONOPERATOR_ANY_NODE_TYPE,
        "NoCondAssign" => ESLINTNOCONDASSIGN_ANY_NODE_TYPE,
        "NoUselessCall" => ESLINTNOUSELESSCALL_ANY_NODE_TYPE,
        "NoAlert" => ESLINTNOALERT_ANY_NODE_TYPE,
        "NoDupeKeys" => ESLINTNODUPEKEYS_ANY_NODE_TYPE,
        "NoUnexpectedMultiline" => ESLINTNOUNEXPECTEDMULTILINE_ANY_NODE_TYPE,
        "NoUselessBackreference" => ESLINTNOUSELESSBACKREFERENCE_ANY_NODE_TYPE,
        "NoConstructorReturn" => ESLINTNOCONSTRUCTORRETURN_ANY_NODE_TYPE,
        "NoCaller" => ESLINTNOCALLER_ANY_NODE_TYPE,
        "MaxDepth" => ESLINTMAXDEPTH_ANY_NODE_TYPE,
        "NoPrototypeBuiltins" => ESLINTNOPROTOTYPEBUILTINS_ANY_NODE_TYPE,
        "UnicodeBom" => ESLINTUNICODEBOM_ANY_NODE_TYPE,
        "DefaultCaseLast" => ESLINTDEFAULTCASELAST_ANY_NODE_TYPE,
        "NoUnreachable" => ESLINTNOUNREACHABLE_ANY_NODE_TYPE,
        "PreferDestructuring" => ESLINTPREFERDESTRUCTURING_ANY_NODE_TYPE,
        "NoNewNativeNonconstructor" => ESLINTNONEWNATIVENONCONSTRUCTOR_ANY_NODE_TYPE,
        "NoUnsafeNegation" => ESLINTNOUNSAFENEGATION_ANY_NODE_TYPE,
        "NoUselessEscape" => ESLINTNOUSELESSESCAPE_ANY_NODE_TYPE,
        "NoLoneBlocks" => ESLINTNOLONEBLOCKS_ANY_NODE_TYPE,
        "ReturnChecker" => ARRAYCALLBACKRETURNRETURNCHECKER_ANY_NODE_TYPE,
        "Mod" => ARRAYCALLBACKRETURNMOD_ANY_NODE_TYPE,
        "NoConstantBinaryExpression" => ESLINTNOCONSTANTBINARYEXPRESSION_ANY_NODE_TYPE,
        "VarsOnTop" => ESLINTVARSONTOP_ANY_NODE_TYPE,
        "MaxLines" => ESLINTMAXLINES_ANY_NODE_TYPE,
        "NoUndefined" => ESLINTNOUNDEFINED_ANY_NODE_TYPE,
        "NoTernary" => ESLINTNOTERNARY_ANY_NODE_TYPE,
        "NoObjCalls" => ESLINTNOOBJCALLS_ANY_NODE_TYPE,
        "NoReturnAssign" => ESLINTNORETURNASSIGN_ANY_NODE_TYPE,
        "NoShadowRestrictedNames" => ESLINTNOSHADOWRESTRICTEDNAMES_ANY_NODE_TYPE,
        "MaxNestedCallbacks" => ESLINTMAXNESTEDCALLBACKS_ANY_NODE_TYPE,
        "NoAsyncPromiseExecutor" => ESLINTNOASYNCPROMISEEXECUTOR_ANY_NODE_TYPE,
        "NoConsole" => ESLINTNOCONSOLE_ANY_NODE_TYPE,
        "NoFallthrough" => ESLINTNOFALLTHROUGH_ANY_NODE_TYPE,
        "NoEmptyFunction" => ESLINTNOEMPTYFUNCTION_ANY_NODE_TYPE,
        "NoUselessCatch" => ESLINTNOUSELESSCATCH_ANY_NODE_TYPE,
        "NoNewWrappers" => ESLINTNONEWWRAPPERS_ANY_NODE_TYPE,
        "RequireAwait" => ESLINTREQUIREAWAIT_ANY_NODE_TYPE,
        "NoThisBeforeSuper" => ESLINTNOTHISBEFORESUPER_ANY_NODE_TYPE,
        "NoExtraBooleanCast" => ESLINTNOEXTRABOOLEANCAST_ANY_NODE_TYPE,
        "NoInvalidRegexp" => ESLINTNOINVALIDREGEXP_ANY_NODE_TYPE,
        "FuncStyle" => ESLINTFUNCSTYLE_ANY_NODE_TYPE,
        "NoLabelVar" => ESLINTNOLABELVAR_ANY_NODE_TYPE,
        "NoProto" => ESLINTNOPROTO_ANY_NODE_TYPE,
        "NoUnsafeOptionalChaining" => ESLINTNOUNSAFEOPTIONALCHAINING_ANY_NODE_TYPE,
        "GuardForIn" => ESLINTGUARDFORIN_ANY_NODE_TYPE,
        "PreferObjectHasOwn" => ESLINTPREFEROBJECTHASOWN_ANY_NODE_TYPE,
        "NoDivRegex" => ESLINTNODIVREGEX_ANY_NODE_TYPE,
        "NoExAssign" => ESLINTNOEXASSIGN_ANY_NODE_TYPE,
        "MaxClassesPerFile" => ESLINTMAXCLASSESPERFILE_ANY_NODE_TYPE,
        "NoContinue" => ESLINTNOCONTINUE_ANY_NODE_TYPE,
        "NoDupeElseIf" => ESLINTNODUPEELSEIF_ANY_NODE_TYPE,
        "ArrowBodyStyle" => ESLINTARROWBODYSTYLE_ANY_NODE_TYPE,
        "NoSelfAssign" => ESLINTNOSELFASSIGN_ANY_NODE_TYPE,
        "JsxNoNewObjectAsProp" => REACTPERFJSXNONEWOBJECTASPROP_ANY_NODE_TYPE,
        "JsxNoNewArrayAsProp" => REACTPERFJSXNONEWARRAYASPROP_ANY_NODE_TYPE,
        "JsxNoNewFunctionAsProp" => REACTPERFJSXNONEWFUNCTIONASPROP_ANY_NODE_TYPE,
        "JsxNoJsxAsProp" => REACTPERFJSXNOJSXASPROP_ANY_NODE_TYPE,
        "MisrefactoredAssignOp" => OXCMISREFACTOREDASSIGNOP_ANY_NODE_TYPE,
        "OnlyUsedInRecursion" => OXCONLYUSEDINRECURSION_ANY_NODE_TYPE,
        "NoAccumulatingSpread" => OXCNOACCUMULATINGSPREAD_ANY_NODE_TYPE,
        "NoOptionalChaining" => OXCNOOPTIONALCHAINING_ANY_NODE_TYPE,
        "NoConstEnum" => OXCNOCONSTENUM_ANY_NODE_TYPE,
        "BadReplaceAllArg" => OXCBADREPLACEALLARG_ANY_NODE_TYPE,
        "BadArrayMethodOnArguments" => OXCBADARRAYMETHODONARGUMENTS_ANY_NODE_TYPE,
        "BadCharAtComparison" => OXCBADCHARATCOMPARISON_ANY_NODE_TYPE,
        "NoBarrelFile" => OXCNOBARRELFILE_ANY_NODE_TYPE,
        "NoRestSpreadProperties" => OXCNORESTSPREADPROPERTIES_ANY_NODE_TYPE,
        "DoubleComparisons" => OXCDOUBLECOMPARISONS_ANY_NODE_TYPE,
        "BadBitwiseOperator" => OXCBADBITWISEOPERATOR_ANY_NODE_TYPE,
        "MissingThrow" => OXCMISSINGTHROW_ANY_NODE_TYPE,
        "ApproxConstant" => OXCAPPROXCONSTANT_ANY_NODE_TYPE,
        "BadMinMaxFunc" => OXCBADMINMAXFUNC_ANY_NODE_TYPE,
        "BadComparisonSequence" => OXCBADCOMPARISONSEQUENCE_ANY_NODE_TYPE,
        "UninvokedArrayCallback" => OXCUNINVOKEDARRAYCALLBACK_ANY_NODE_TYPE,
        "ErasingOp" => OXCERASINGOP_ANY_NODE_TYPE,
        "NumberArgOutOfRange" => OXCNUMBERARGOUTOFRANGE_ANY_NODE_TYPE,
        "ConstComparisons" => OXCCONSTCOMPARISONS_ANY_NODE_TYPE,
        "NoAsyncEndpointHandlers" => OXCNOASYNCENDPOINTHANDLERS_ANY_NODE_TYPE,
        "BadObjectLiteralComparison" => OXCBADOBJECTLITERALCOMPARISON_ANY_NODE_TYPE,
        "NoAsyncAwait" => OXCNOASYNCAWAIT_ANY_NODE_TYPE,
        "NoMapSpread" => OXCNOMAPSPREAD_ANY_NODE_TYPE,
        "JsxNoScriptUrl" => REACTJSXNOSCRIPTURL_ANY_NODE_TYPE,
        "CheckedRequiresOnchangeOrReadonly" => REACTCHECKEDREQUIRESONCHANGEORREADONLY_ANY_NODE_TYPE,
        "RulesOfHooks" => REACTRULESOFHOOKS_ANY_NODE_TYPE,
        "JsxNoUndef" => REACTJSXNOUNDEF_ANY_NODE_TYPE,
        "JsxFragments" => REACTJSXFRAGMENTS_ANY_NODE_TYPE,
        "JsxCurlyBracePresence" => REACTJSXCURLYBRACEPRESENCE_ANY_NODE_TYPE,
        "RequireRenderReturn" => REACTREQUIRERENDERRETURN_ANY_NODE_TYPE,
        "NoStringRefs" => REACTNOSTRINGREFS_ANY_NODE_TYPE,
        "JsxNoDuplicateProps" => REACTJSXNODUPLICATEPROPS_ANY_NODE_TYPE,
        "StylePropObject" => REACTSTYLEPROPOBJECT_ANY_NODE_TYPE,
        "NoChildrenProp" => REACTNOCHILDRENPROP_ANY_NODE_TYPE,
        "ExhaustiveDeps" => REACTEXHAUSTIVEDEPS_ANY_NODE_TYPE,
        "NoDirectMutationState" => REACTNODIRECTMUTATIONSTATE_ANY_NODE_TYPE,
        "JsxPropsNoSpreadMulti" => REACTJSXPROPSNOSPREADMULTI_ANY_NODE_TYPE,
        "JsxBooleanValue" => REACTJSXBOOLEANVALUE_ANY_NODE_TYPE,
        "IframeMissingSandbox" => REACTIFRAMEMISSINGSANDBOX_ANY_NODE_TYPE,
        "NoDangerWithChildren" => REACTNODANGERWITHCHILDREN_ANY_NODE_TYPE,
        "JsxNoTargetBlank" => REACTJSXNOTARGETBLANK_ANY_NODE_TYPE,
        "NoIsMounted" => REACTNOISMOUNTED_ANY_NODE_TYPE,
        "PreferEs6Class" => REACTPREFERES6CLASS_ANY_NODE_TYPE,
        "ForbidElements" => REACTFORBIDELEMENTS_ANY_NODE_TYPE,
        "JsxFilenameExtension" => REACTJSXFILENAMEEXTENSION_ANY_NODE_TYPE,
        "VoidDomElementsNoChildren" => REACTVOIDDOMELEMENTSNOCHILDREN_ANY_NODE_TYPE,
        "NoSetState" => REACTNOSETSTATE_ANY_NODE_TYPE,
        "ButtonHasType" => REACTBUTTONHASTYPE_ANY_NODE_TYPE,
        "NoUnescapedEntities" => REACTNOUNESCAPEDENTITIES_ANY_NODE_TYPE,
        "NoFindDomNode" => REACTNOFINDDOMNODE_ANY_NODE_TYPE,
        "ReactInJsxScope" => REACTREACTINJSXSCOPE_ANY_NODE_TYPE,
        "SelfClosingComp" => REACTSELFCLOSINGCOMP_ANY_NODE_TYPE,
        "JsxNoUselessFragment" => REACTJSXNOUSELESSFRAGMENT_ANY_NODE_TYPE,
        "ForwardRefUsesRef" => REACTFORWARDREFUSESREF_ANY_NODE_TYPE,
        "JsxKey" => REACTJSXKEY_ANY_NODE_TYPE,
        "NoNamespace" => REACTNONAMESPACE_ANY_NODE_TYPE,
        "JsxNoCommentTextnodes" => REACTJSXNOCOMMENTTEXTNODES_ANY_NODE_TYPE,
        "NoRenderReturnValue" => REACTNORENDERRETURNVALUE_ANY_NODE_TYPE,
        "NoDanger" => REACTNODANGER_ANY_NODE_TYPE,
        "NoArrayIndexKey" => REACTNOARRAYINDEXKEY_ANY_NODE_TYPE,
        "NoUnknownProperty" => REACTNOUNKNOWNPROPERTY_ANY_NODE_TYPE,
        "NoMagicArrayFlatDepth" => UNICORNNOMAGICARRAYFLATDEPTH_ANY_NODE_TYPE,
        "PreferObjectFromEntries" => UNICORNPREFEROBJECTFROMENTRIES_ANY_NODE_TYPE,
        "PreferEventTarget" => UNICORNPREFEREVENTTARGET_ANY_NODE_TYPE,
        "NumberLiteralCase" => UNICORNNUMBERLITERALCASE_ANY_NODE_TYPE,
        "NoThisAssignment" => UNICORNNOTHISASSIGNMENT_ANY_NODE_TYPE,
        "PreferSetHas" => UNICORNPREFERSETHAS_ANY_NODE_TYPE,
        "PreferArrayFind" => UNICORNPREFERARRAYFIND_ANY_NODE_TYPE,
        "NoLonelyIf" => UNICORNNOLONELYIF_ANY_NODE_TYPE,
        "NoTypeofUndefined" => UNICORNNOTYPEOFUNDEFINED_ANY_NODE_TYPE,
        "PreferSetSize" => UNICORNPREFERSETSIZE_ANY_NODE_TYPE,
        "PreferMathMinMax" => UNICORNPREFERMATHMINMAX_ANY_NODE_TYPE,
        "PreferStringTrimStartEnd" => UNICORNPREFERSTRINGTRIMSTARTEND_ANY_NODE_TYPE,
        "PreferNodeProtocol" => UNICORNPREFERNODEPROTOCOL_ANY_NODE_TYPE,
        "NoUselessSwitchCase" => UNICORNNOUSELESSSWITCHCASE_ANY_NODE_TYPE,
        "NoAnonymousDefaultExport" => UNICORNNOANONYMOUSDEFAULTEXPORT_ANY_NODE_TYPE,
        "RequireArrayJoinSeparator" => UNICORNREQUIREARRAYJOINSEPARATOR_ANY_NODE_TYPE,
        "PreferStringReplaceAll" => UNICORNPREFERSTRINGREPLACEALL_ANY_NODE_TYPE,
        "FilenameCase" => UNICORNFILENAMECASE_ANY_NODE_TYPE,
        "NoAccessorRecursion" => UNICORNNOACCESSORRECURSION_ANY_NODE_TYPE,
        "NoNewArray" => UNICORNNONEWARRAY_ANY_NODE_TYPE,
        "PreferModernMathApis" => UNICORNPREFERMODERNMATHAPIS_ANY_NODE_TYPE,
        "NoUselessFallbackInSpread" => UNICORNNOUSELESSFALLBACKINSPREAD_ANY_NODE_TYPE,
        "ConsistentEmptyArraySpread" => UNICORNCONSISTENTEMPTYARRAYSPREAD_ANY_NODE_TYPE,
        "ConsistentExistenceIndexCheck" => UNICORNCONSISTENTEXISTENCEINDEXCHECK_ANY_NODE_TYPE,
        "EmptyBraceSpaces" => UNICORNEMPTYBRACESPACES_ANY_NODE_TYPE,
        "NoArrayMethodThisArgument" => UNICORNNOARRAYMETHODTHISARGUMENT_ANY_NODE_TYPE,
        "PreferStringRaw" => UNICORNPREFERSTRINGRAW_ANY_NODE_TYPE,
        "ConsistentDateClone" => UNICORNCONSISTENTDATECLONE_ANY_NODE_TYPE,
        "ConsistentAssert" => UNICORNCONSISTENTASSERT_ANY_NODE_TYPE,
        "NoInvalidFetchOptions" => UNICORNNOINVALIDFETCHOPTIONS_ANY_NODE_TYPE,
        "PreferSpread" => UNICORNPREFERSPREAD_ANY_NODE_TYPE,
        "PreferDomNodeAppend" => UNICORNPREFERDOMNODEAPPEND_ANY_NODE_TYPE,
        "PreferArrayIndexOf" => UNICORNPREFERARRAYINDEXOF_ANY_NODE_TYPE,
        "PreferOptionalCatchBinding" => UNICORNPREFEROPTIONALCATCHBINDING_ANY_NODE_TYPE,
        "SwitchCaseBraces" => UNICORNSWITCHCASEBRACES_ANY_NODE_TYPE,
        "PreferArraySome" => UNICORNPREFERARRAYSOME_ANY_NODE_TYPE,
        "NoNewBuffer" => UNICORNNONEWBUFFER_ANY_NODE_TYPE,
        "PreferNativeCoercionFunctions" => UNICORNPREFERNATIVECOERCIONFUNCTIONS_ANY_NODE_TYPE,
        "NoDocumentCookie" => UNICORNNODOCUMENTCOOKIE_ANY_NODE_TYPE,
        "NoNestedTernary" => UNICORNNONESTEDTERNARY_ANY_NODE_TYPE,
        "NoZeroFractions" => UNICORNNOZEROFRACTIONS_ANY_NODE_TYPE,
        "NoAwaitInPromiseMethods" => UNICORNNOAWAITINPROMISEMETHODS_ANY_NODE_TYPE,
        "TextEncodingIdentifierCase" => UNICORNTEXTENCODINGIDENTIFIERCASE_ANY_NODE_TYPE,
        "ThrowNewError" => UNICORNTHROWNEWERROR_ANY_NODE_TYPE,
        "NoUselessLengthCheck" => UNICORNNOUSELESSLENGTHCHECK_ANY_NODE_TYPE,
        "ConsistentFunctionScoping" => UNICORNCONSISTENTFUNCTIONSCOPING_ANY_NODE_TYPE,
        "PreferIncludes" => UNICORNPREFERINCLUDES_ANY_NODE_TYPE,
        "PreferPrototypeMethods" => UNICORNPREFERPROTOTYPEMETHODS_ANY_NODE_TYPE,
        "NoNegationInEqualityCheck" => UNICORNNONEGATIONINEQUALITYCHECK_ANY_NODE_TYPE,
        "ExplicitLengthCheck" => UNICORNEXPLICITLENGTHCHECK_ANY_NODE_TYPE,
        "NoArrayReduce" => UNICORNNOARRAYREDUCE_ANY_NODE_TYPE,
        "RequirePostMessageTargetOrigin" => UNICORNREQUIREPOSTMESSAGETARGETORIGIN_ANY_NODE_TYPE,
        "PreferArrayFlat" => UNICORNPREFERARRAYFLAT_ANY_NODE_TYPE,
        "CatchErrorName" => UNICORNCATCHERRORNAME_ANY_NODE_TYPE,
        "NoEmptyFile" => UNICORNNOEMPTYFILE_ANY_NODE_TYPE,
        "NoUnnecessaryAwait" => UNICORNNOUNNECESSARYAWAIT_ANY_NODE_TYPE,
        "NoUnnecessarySliceEnd" => UNICORNNOUNNECESSARYSLICEEND_ANY_NODE_TYPE,
        "PreferArrayFlatMap" => UNICORNPREFERARRAYFLATMAP_ANY_NODE_TYPE,
        "NoInvalidRemoveEventListener" => UNICORNNOINVALIDREMOVEEVENTLISTENER_ANY_NODE_TYPE,
        "NumericSeparatorsStyle" => UNICORNNUMERICSEPARATORSSTYLE_ANY_NODE_TYPE,
        "NoInstanceofBuiltins" => UNICORNNOINSTANCEOFBUILTINS_ANY_NODE_TYPE,
        "NoThenable" => UNICORNNOTHENABLE_ANY_NODE_TYPE,
        "NoProcessExit" => UNICORNNOPROCESSEXIT_ANY_NODE_TYPE,
        "NoAbusiveEslintDisable" => UNICORNNOABUSIVEESLINTDISABLE_ANY_NODE_TYPE,
        "PreferReflectApply" => UNICORNPREFERREFLECTAPPLY_ANY_NODE_TYPE,
        "PreferStructuredClone" => UNICORNPREFERSTRUCTUREDCLONE_ANY_NODE_TYPE,
        "PreferDomNodeTextContent" => UNICORNPREFERDOMNODETEXTCONTENT_ANY_NODE_TYPE,
        "NewForBuiltins" => UNICORNNEWFORBUILTINS_ANY_NODE_TYPE,
        "NoLengthAsSliceEnd" => UNICORNNOLENGTHASSLICEEND_ANY_NODE_TYPE,
        "NoHexEscape" => UNICORNNOHEXESCAPE_ANY_NODE_TYPE,
        "PreferQuerySelector" => UNICORNPREFERQUERYSELECTOR_ANY_NODE_TYPE,
        "PreferNegativeIndex" => UNICORNPREFERNEGATIVEINDEX_ANY_NODE_TYPE,
        "PreferTypeError" => UNICORNPREFERTYPEERROR_ANY_NODE_TYPE,
        "PreferBlobReadingMethods" => UNICORNPREFERBLOBREADINGMETHODS_ANY_NODE_TYPE,
        "NoNull" => UNICORNNONULL_ANY_NODE_TYPE,
        "PreferStringSlice" => UNICORNPREFERSTRINGSLICE_ANY_NODE_TYPE,
        "PreferMathTrunc" => UNICORNPREFERMATHTRUNC_ANY_NODE_TYPE,
        "NoUnreadableArrayDestructuring" => UNICORNNOUNREADABLEARRAYDESTRUCTURING_ANY_NODE_TYPE,
        "PreferCodePoint" => UNICORNPREFERCODEPOINT_ANY_NODE_TYPE,
        "NoConsoleSpaces" => UNICORNNOCONSOLESPACES_ANY_NODE_TYPE,
        "PreferStringStartsEndsWith" => UNICORNPREFERSTRINGSTARTSENDSWITH_ANY_NODE_TYPE,
        "NoSinglePromiseInPromiseMethods" => UNICORNNOSINGLEPROMISEINPROMISEMETHODS_ANY_NODE_TYPE,
        "PreferNumberProperties" => UNICORNPREFERNUMBERPROPERTIES_ANY_NODE_TYPE,
        "NoAwaitExpressionMember" => UNICORNNOAWAITEXPRESSIONMEMBER_ANY_NODE_TYPE,
        "NoInstanceofArray" => UNICORNNOINSTANCEOFARRAY_ANY_NODE_TYPE,
        "ErrorMessage" => UNICORNERRORMESSAGE_ANY_NODE_TYPE,
        "PreferDomNodeRemove" => UNICORNPREFERDOMNODEREMOVE_ANY_NODE_TYPE,
        "PreferAddEventListener" => UNICORNPREFERADDEVENTLISTENER_ANY_NODE_TYPE,
        "PreferDateNow" => UNICORNPREFERDATENOW_ANY_NODE_TYPE,
        "NoUselessUndefined" => UNICORNNOUSELESSUNDEFINED_ANY_NODE_TYPE,
        "NoStaticOnlyClass" => UNICORNNOSTATICONLYCLASS_ANY_NODE_TYPE,
        "NoUselessPromiseResolveReject" => UNICORNNOUSELESSPROMISERESOLVEREJECT_ANY_NODE_TYPE,
        "EscapeCase" => UNICORNESCAPECASE_ANY_NODE_TYPE,
        "PreferLogicalOperatorOverTernary" => UNICORNPREFERLOGICALOPERATOROVERTERNARY_ANY_NODE_TYPE,
        "NoObjectAsDefaultParameter" => UNICORNNOOBJECTASDEFAULTPARAMETER_ANY_NODE_TYPE,
        "NoUnnecessaryArrayFlatDepth" => UNICORNNOUNNECESSARYARRAYFLATDEPTH_ANY_NODE_TYPE,
        "ConstEval" => NOUSELESSSPREADCONSTEVAL_ANY_NODE_TYPE,
        "Mod" => NOUSELESSSPREADMOD_ANY_NODE_TYPE,
        "PreferDomNodeDataset" => UNICORNPREFERDOMNODEDATASET_ANY_NODE_TYPE,
        "NoUnreadableIife" => UNICORNNOUNREADABLEIIFE_ANY_NODE_TYPE,
        "PreferModernDomApis" => UNICORNPREFERMODERNDOMAPIS_ANY_NODE_TYPE,
        "NoArrayForEach" => UNICORNNOARRAYFOREACH_ANY_NODE_TYPE,
        "PreferGlobalThis" => UNICORNPREFERGLOBALTHIS_ANY_NODE_TYPE,
        "PreferRegexpTest" => UNICORNPREFERREGEXPTEST_ANY_NODE_TYPE,
        "RequireNumberToFixedDigitsArgument" => UNICORNREQUIRENUMBERTOFIXEDDIGITSARGUMENT_ANY_NODE_TYPE,
        _ => true, // Fallback for unknown rules - run on all nodes
    }
}
