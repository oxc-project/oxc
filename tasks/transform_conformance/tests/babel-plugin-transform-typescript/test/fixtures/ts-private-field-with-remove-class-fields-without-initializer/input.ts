export class ArrayBufferViewTransferable implements Transferable {
  #view: ArrayBufferView;
  constructor(view: ArrayBufferView) {
    this.#view = view;
  }

  get [kTransferable](): object {
    return this.#view.buffer;
  }

  get [kValue](): object {
    return this.#view;
  }
}
