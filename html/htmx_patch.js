htmx.config.methodsThatUseUrlParams = ["get"];
// Ensure that null values aren't sent as strings to avoid deserialization issues
// because in htmx internals they are transformed to FormData entries and become strings
htmx.defineExtension('no-nulls', {
    onEvent: function (name, evt) {
        if (name === 'htmx:configRequest') {
            const params = evt.detail.parameters;
            for (const [key, value] of params.entries()) {
                if (value === "null") params.delete(key);
            }
        }
    },
});
document.body.setAttribute('hx-ext', (document.body.getAttribute('hx-ext') || '') + ', no-nulls');
