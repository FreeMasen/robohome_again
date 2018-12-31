
window.addEventListener('DOMContentLoaded', async () => {
    try {
        let dh = await import('./robohome_crypto');
        auth(dh);
    } catch (e) {
        console.error(e);
    }
});
/**
 * @param {WebAssembly.Module} dh The web assembly module for doing DH
 */
async function auth(dh) {
    let pair = dh.gen_pair();
    let pub = pair.public();
    localStorage.setItem('public-key', stringifyBuf(pub));
    let authToken;
    try {
        authToken = await sendPublicKey(pub);
    } catch (e) {
        return console.error(e);
    }
    localStorage.setItem('auth-token', authToken);
    window.location = '/';
}
/**
 * Exchange public keys
 * @param {Uint8Array} pub The public key to send to the server
 * @returns {Promise<string>}
 */
async function sendPublicKey(pub) {
    let res;
    let search = location.search;
    if (search.startsWith('?')) {
        search = search.substr(1);
    }
    let authToken = search.split('&').map(pair => pair.split('=')).filter(pair => pair[0] === 'token')[0][1];
    if (!authToken || authToken == '') {
        throw new Error('Failed to authorize this device');
    }
    try {
        res = await fetch('/key-exchange', {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`
            },
            body: pub
        });
    } catch (e) {
        console.error('Unable to execute key exchange');
        throw e;
    }
    if (!res.ok) {
        let body = await res.json();
        throw new Error(`Failed to get public key: ${body.message}`);
    }
    return authToken;
}

function stringifyBuf(buf) {
    let ret = '';
    buf.forEach(n => ret += `0${n.toString(16)}`.substr(-2));
    return ret;
}

function bufifyString(s) {
    if (s.length != 64) {
        throw new Error('Unable to create buffer, string must be 64 characters long');
    }
    let ret = new Uint8Array(32);
    let counter = 0;
    for (var i = 0; i < 64; i += 2) {
        ret[counter] = parseInt(s.substr(i, 2), 16);
        counter += 1;
    }
    return ret;
}