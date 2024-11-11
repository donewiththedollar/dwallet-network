/// This module contains the secp256k1 constants for the class groups protocol.
/// NOTE: This is a temporary solution until the class groups DKG is complete.
/// Todo (#312): Remove this module and use the class groups DKG to generate the constants.
use group::secp256k1;
use homomorphic_encryption::AdditivelyHomomorphicDecryptionKey;

const PUBLIC_PARAMETERS : &str = "3a5468652066696e697465206669656c64206f6620696e746567657273206d6f64756c6f207072696d65207120245c6d61746862627b5a7d5f7124414136d08c5ed2bf3ba048afe6dcaebafeffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000000109536563703235366b310b5765696572737472617373414136d08c5ed2bf3ba048afe6dcaebafeffffffffffffffffffffffffffffff2ffcfffffeffffffffffffffffffffffffffffffffffffffffffffffffffffff210279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798000000000000000000000000000000000000000000000000000000000000000007000000000000000000000000000000000000000000000000000000000000003a5468652066696e697465206669656c64206f6620696e746567657273206d6f64756c6f207072696d65207120245c6d61746862627b5a7d5f7124414136d08c5ed2bf3ba048afe6dcaebafeffffffffffffffffffffffffffffff00000000000000000000000000000000000000000000000000000000000000010008000000000000ea020000000000006b04000000000000df4ea16370272c6a439701b05e578271ef1e67656e1291461884f9f1abf8fb2962d7206d48cb5ec1507437d8925606c71ed9b14f916c4e77a3a5d5761f509c3e040bde65e87a78e53adc1f4dbc8b411a71ff5e9698234ebc2a297931a01e71b92cabb39f8b86d88f9fb17d53543a90434551ebb35f37f5b87e3d1b39760fcf00126e9df86cc6495807a4ede705153dee2ba6304c005718bc167f7fdd60c84c1a04efdafe7c5f24f271a4bdeaf24f6f40c6348b329214317e67fb53cd31260b526921089bb0a7c9a7f66c27f32edc5ea3def5f609104ba577d4bfd9139b65fda37555b9ef74e3e6a70a00000000000000000000000000000000000000000000000200000000000000011d83b210defd2b6de01287e01906a73c34f162538bdaeb6b2375672ab1f773ccc1e516c8fd904a23a837e484de4b7bbf3aaa7f4b4ed3ba4fcb0dc4777bd2686c07e9f7db04ac796c908fedadce91331538794b3a4e56ed989cf5d7c21dbb003feb2776e50de78355098ecd347f55fb541d37f88939ff1dc5d2534c9a964c90852a4f4930027f58edad2f10925198895aefdc95900aa14f61daae75ad24b671c345ea2ab9b19b3f050156895443b6ffb1599e914780edd42effcb450aaaeaa2193eca9378966faa73f284f6dfdbc5363276664623e85c432b7b2c6620bcb7633b069d6240efff24de026b2b011e8a84ba23f503d700d68f8241594bc9e8be43f27712afa3bf8e3cd29cb4f2a7fda75f48910ce86c9f2a4663b568abf3c28eccf3f286b6ab328230b1ce26d83dfdf9549711552c216ad257c437a45e2c0abdaf7c215ca94874f3040c0cb58b0475261402e6f597f70ccda5cb94553e592d7eb768e20b000000ff3bdf4ea16370272c6a439701b05e578271ef1e67656e1291461884f9f1abf8fb2962d7206d48cb5ec1507437d8925606c71ed9b14f916c4e77a3a5d5761f509c3e040bde65e87a78e53adc1f4dbc8b411a71ff5e9698234ebc2a297931a01e71b92cabb39f8b86d88f9fb17d53543a90434551ebb35f37f5b87e3d1b39760fcf00126e9df86cc6495807a4ede705153dee2ba6304c005718bc167f7fdd60c84c1a04efdafe7c5f24f271a4bdeaf24f6f40c6348b329214317e67fb53cd31260b526921089bb0a7c9a7f66c27f32edc5ea3def5f609104ba577d4bfd9139b65fda37555b9ef74e3e6a70a000000015ff17d4ed5683f3adbc05947ff52cd8e3bafce24536877d39c6ec9e8d935c1c0b82e0d72457510ea1525f8acfc91d776de4d8fbed1cdd4b1b122778fe78f2064fc097614e40fa631100f2937178217f4430b1ea7011380f5d091686f02deef126fba54e41fc995af299795b70744aae573b0670afbfba08164d1b5d44b4420ae47c4d78c621537a910d1d2536071e2febcf52188b7b4b68cefbfd9139b65fda37555b9ef74e3e6a70a00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001df4ea16370272c6a439701b05e578271ef1e67656e1291461884f9f1abf8fb2962d7206d48cb5ec1507437d8925606c71ed9b14f916c4e77a3a5d5761f509c3e040bde65e87a78e53adc1f4dbc8b411a71ff5e9698234ebc2a297931a01e71b92cabb39f8b86d88f9fb17d53543a90434551ebb35f37f5b87e3d1b39760fcf00126e9df86cc6495807a4ede705153dee2ba6304c005718bc167f7fdd60c84c1a04efdafe7c5f24f271a4bdeaf24f6f40c6348b329214317e67fb53cd31260b526921089bb0a7c9a7f66c27f32edc5ea3def5f609104ba577d4bfd9139b65fda37555b9ef74e3e6a70a0000000000000000000000000000000000000000000000414136d08c5ed2bf3ba048afe6dcaebafeffffffffffffffffffffffffffffff9feaa6d55567197ea3515f1636b6ed1562996b4fe19aaf3c0fa1cfdd40c5333afad9aa973c1abd27a20554bd1ae7de19ae8f6d81ae18250c002d981cdeff7258067b43bea531c996d2af350e1543dca33bdd83a5b8ef45a8fb0d431b4b7a0ce2c7da38ce6129ad7937832804f93ba42cac7537478b693f17fdbfd9139b65fda37555b9ef74e3e6a70a000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001414136d08c5ed2bf3ba048afe6dcaebafeffffffffffffffffffffffffffffff000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000081120e38a1503f49b498c5bda873c32ec507cd5be4f597e6c59bc681d51c679d83826ca019bda47f7740915ecdb95d75fdffffffffffffffffffffffffffffff0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000007000000000000000011081120e38a1503f49b498c5bda873c32ec507cd5be4f597e6c59bc681d51c679d83826ca019bda47f7740915ecdb95d75fdffffffffffffffffffffffffffffff0108414136d08c5ed2bf3ba048afe6dcaebafeffffffffffffffffffffffffffffff012b587c9f5335da8fce3670d6d1bf54b3e3ceab33c914dadd34a75b327a764d3030ae4b835c511d847a45093e2b7fe4b59d77d3a36f7433756cacc8dde3f92308197f821d05f983690cc443cacd85e005fdd082c769c004603d7424da9b80f7bbc49b2e15f94772e56bca65e5ed01916af91cec99c2fe3e682059742df5121188eb11f135a358c54d2a44b4f414589cb83f6f7d08e22dad2de3fb6ff6c46659ff685d55ee3bddb8f9a902000000ff3bdf4ea16370272c6a439701b05e578271ef1e67656e1291461884f9f1abf8fb2962d7206d48cb5ec1507437d8925606c71ed9b14f916c4e77a3a5d5761f509c3e040bde65e87a78e53adc1f4dbc8b411a71ff5e9698234ebc2a297931a01e71b92cabb39f8b86d88f9fb17d53543a90434551ebb35f37f5b87e3d1b39760fcf00126e9df86cc6495807a4ede705153dee2ba6304c005718bc167f7fdd60c84c1a04efdafe7c5f24f271a4bdeaf24f6f40c6348b329214317e67fb53cd31260b526921089bb0a7c9a7f66c27f32edc5ea3def5f609104ba577d4bfd9139b65fda37555b9ef74e3e6a70a000000011d6070bdd5c7ab5cd7a930d18bf6915fb5beccfa56d3f7a77aa54c875e98daff0e5fe9bfd96ecd4802ba3fab7a7bc0eea04cc30232f85b38ed7d9650b64f872ef76339c4c77893666f282c2d251c912f42c20da64ef568189fe3987eb851a2bae3342c7547833c770a8a9e0c7c0112dec40f8e4c82011d2ff89b85e83e09b74993b780ad1977c38b6e217747bf9893a68d6ea7ad76a6ade9f674c0e64a2e0c0768880ff8ff3568f0bc838a68b92d789fc80f4171ae6b3c7d8aed478a31d47c837cbe9f3e3d8a3f0258309a6771f047edb9ea0300fe5ca46511b77f958ea2cc010876af61c77a3f5d36f624011ec5dbdd69ff06beb401b4a21dd220a166f53abd974fb5b60b7482105715c9a7aea315c095f212893c627094edffe7482bd1b5f880b8d0e3f4392edcb47002502cfa09eab5fe623fd5e675297a7ce760ab786f98c08e11b89afad21e8c24b6113a77ab1c3836fa0fbc7c9f0f0317469e728f92813e05000000ff3bdf4ea16370272c6a439701b05e578271ef1e67656e1291461884f9f1abf8fb2962d7206d48cb5ec1507437d8925606c71ed9b14f916c4e77a3a5d5761f509c3e040bde65e87a78e53adc1f4dbc8b411a71ff5e9698234ebc2a297931a01e71b92cabb39f8b86d88f9fb17d53543a90434551ebb35f37f5b87e3d1b39760fcf00126e9df86cc6495807a4ede705153dee2ba6304c005718bc167f7fdd60c84c1a04efdafe7c5f24f271a4bdeaf24f6f40c6348b329214317e67fb53cd31260b526921089bb0a7c9a7f66c27f32edc5ea3def5f609104ba577d4bfd9139b65fda37555b9ef74e3e6a70a000000";
const INNER_SECRET : &str = "1e7983524ed58b93fa4f6d9f1f7cd4feba69f779d15169626c7b3f28b0ba1a24cfa42d86af0d5b52dead2b615beaeff928f9439184dc534685d989d78ec5a3bd8366e54afa8c219c411a415a4620e1d2c30ce9e54e4634344df8144a5c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

pub fn protocol_public_parameters() -> twopc_mpc::class_groups::ProtocolPublicParameters<
    { secp256k1::SCALAR_LIMBS },
    { twopc_mpc::secp256k1::class_groups::DISCRIMINANT_LIMBS },
    secp256k1::GroupElement,
> {
    // Safe to unwrap as we're using a hardcoded constant.
    let protocol_public_parameters = hex::decode(&PUBLIC_PARAMETERS).unwrap();
    bcs::from_bytes(&protocol_public_parameters).unwrap()
}

pub fn decryption_key() -> twopc_mpc::secp256k1::class_groups::DecryptionKey {
    let secret_key: class_groups::test_helpers::SecretKey<
        { twopc_mpc::secp256k1::class_groups::DISCRIMINANT_LIMBS },
    > = bcs::from_bytes(&hex::decode(&INNER_SECRET).unwrap()).unwrap();
    let decryption_key =
        <twopc_mpc::secp256k1::class_groups::DecryptionKey as AdditivelyHomomorphicDecryptionKey<
            { secp256k1::SCALAR_LIMBS },
            twopc_mpc::secp256k1::class_groups::EncryptionKey,
        >>::new(
            secret_key,
            &protocol_public_parameters().encryption_scheme_public_parameters,
        )
        .unwrap();
    decryption_key
}
