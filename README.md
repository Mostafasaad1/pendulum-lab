# 🪄 Pendulum Lab — Rust Physics Simulation Collection

A collection of **physics simulation projects written in Rust**, exploring the fascinating dynamics of pendulums — from simple single-body motion to complex multi-link and wave systems.

This repository represents my **learning journey with Rust**, focusing on:
- Numerical solvers and integration methods  
- Real-time visualization using Rust GUIs  
- Clean modular project organization  

---

## 🧩 Project Structure

```

.
├── single-pendulum       # Basic pendulum physics simulation
│   ├── Cargo.toml
│   └── src/main.rs
│
├── n-pendulum            # Generalized N-link pendulum solver + GUI
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── gui.rs
│       ├── pendulum.rs
│       └── solver.rs
│
└── pendulum_wave         # Visual pendulum wave demo (synchronized oscillations)
├── Cargo.toml
└── src/main.rs

```

---

## ⚙️ Features

- 🧮 **Numerical Integration:** Simulates pendulum motion using physics-based equations.  
- 🎨 **Interactive GUI:** Built with [egui](https://github.com/emilk/egui) for live visualization.  
- 🔗 **Modular Design:** Separate crates for each project, demonstrating good Rust workspace practices.  
- 📚 **Learning-Focused:** Each module reflects progress in mastering Rust’s ecosystem, concurrency, and graphics handling.

---

## 🚀 Getting Started

### 1. Clone the repository
```bash
git clone https://github.com/Mostafasaad1/pendulum-lab.git
cd pendulum-lab
````

### 2. Run any project

#### Single Pendulum

```bash
cargo -vv run -p single-pendulum
```

#### N-Pendulum (with GUI)

```bash
cargo -vv run -p n-pendulum
```

#### Pendulum Wave

```bash
cargo -vv run -p pendulum_wave
```

---

## 🧠 Concepts Covered

* Rust project and workspace management
* GUI programming with **egui** and **eframe**
* Implementing **Euler** and **Runge-Kutta** solvers
* Modeling **multi-body dynamics**
* Real-time physics simulation and visualization

---

## 🧰 Dependencies

* [egui](https://github.com/emilk/egui) — GUI framework for Rust
* [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) — Application framework for egui
* [nalgebra](https://nalgebra.org/) — Linear algebra and matrix operations

(See each project’s `Cargo.toml` for specific versions)

---

## 🎯 Purpose

This repository was built as part of my personal **Rust learning path** — experimenting with numerical physics and GUI-based visualization to strengthen understanding of both **systems programming** and **simulation modeling**.

---

## 🌟 Acknowledgments

Special thanks to the open-source Rust community and the maintainers of **egui** and **nalgebra** for making such powerful tools accessible for learning and experimentation.