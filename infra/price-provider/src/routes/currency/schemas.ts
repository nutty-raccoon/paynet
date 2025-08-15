export const delCurrencySchema = {
    body: {
        type: 'object',
        required: ['currency'],
        properties: {
            currency: { type: 'string' }
        }
    }
}

export const addCurrencySchema = {
    body: {
        type: 'object',
        required: ['currency'],
        properties: {
            currency: { type: 'string' }
        }
    }
}