// case from @types/react-test-renderer

export {};

export interface ReactTestRendererJSON {
    type: string;
    props: { [propName: string]: any };
    children: null | ReactTestRendererNode[];
}
export type ReactTestRendererNode = ReactTestRendererJSON | string;
export interface ReactTestRendererTree extends ReactTestRendererJSON {
    nodeType: 'component' | 'host';
    instance: any;
    rendered: null | ReactTestRendererTree | ReactTestRendererTree[];
}

export function create(nextElement: any, options?: any): any;

export function act(callback: () => Promise<any>): Promise<undefined>;
