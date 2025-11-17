expect(String.raw`</script>` === String.raw`</script>`).toBe(true);
expect(String.raw`</script>` !== String.raw`<\/script>`).toBe(true);
