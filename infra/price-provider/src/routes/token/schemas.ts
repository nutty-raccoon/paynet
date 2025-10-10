export const addTokenSchema = {
  body: {
        type: 'object',
        required: ['address', 'chain'],
        properties: {
            address: { type: 'string' },
            chain: { type: 'string' }
        }
    }
};

export const delTokenSchema = {
    body: {
        type: 'object',
        required: ['address', 'chain', 'symbol'],
        properties: {
            address: { type: 'string' },
            chain: { type: 'string' },
            symbol: {type: 'string'}
        }
    }
};