use terrakit_algorithms::scattering::{
    scatter_poisson,
    PoissonSettings,
    ScatterArea,
};

#[test]
fn test_poisson_scattering() {
    let area = ScatterArea::new(
        0.0,
        100.0,
        0.0,
        100.0,
    );

    let settings = PoissonSettings {
        seed: 12345,
        radius: 5.0,
        attempts: 30,
        prototype_id: 0,
        min_scale: 0.8,
        max_scale: 1.3,
        random_rotation: true,
    };

    let points = scatter_poisson(
        settings,
        area,
    ).unwrap();

    assert!(!points.is_empty());

    println!("Generated {} points", points.len());
}