use crate::task::cluster::ins::Type;

use super::*;

#[test]
fn test_cluster_sorted_after_from() {
    let cluster = Cluster::try_from(
        std::fs::read("test/resources/test-sort-cluster.genin.yaml")
            .unwrap()
            .as_slice(),
    )
    .unwrap();

    println!(
        "test_cluster_sorted_after_from: {:?}",
        cluster
            .topology()
            .iter()
            .map(|instance| &instance.itype)
            .collect::<Vec<&Type>>()
    );

    assert_eq!(&cluster.topology()[0].itype, &Type::Router);
    assert_eq!(&cluster.topology()[1].itype, &Type::Storage);
    assert_eq!(&cluster.topology()[2].itype, &Type::Storage);
    assert_eq!(&cluster.topology()[3].itype, &Type::Custom);
    assert_eq!(&cluster.topology()[4].itype, &Type::Custom);
}
