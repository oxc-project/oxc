const component = {
  methods: {
    async loadMoreData() {
      if (!this.hasMoreData) return
      await this.addData()
    }
  }
}

export default component
