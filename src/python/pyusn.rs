use cpython::{PyObject, PyResult, Python, PyTuple, PyDict};
use usnpkg::usn;

py_class!(class PyUsnConnection |py| {
    def __new__(_cls) -> PyResult<PyUsnConnection> {
        PyUsnConnection::create_instance(py)
    }
});

// add bindings to the generated python module
py_module_initializer!(rustyusn, initrustyusn, PyInit_rustyusn, |py, m| {
    m.add(py, "__doc__", "This module is implemented in Rust.")?;
    m.add(py, "open_file", py_fn!(py, open_file_py(filename: &str)))?;
    Ok(())
});

fn open_file_py(py: Python,filename: &str) -> PyResult<PyObject> {
    println!("Opening {}",filename);

    // get UsnConnection from a filename
    let mut usn_connection = usn::open_file(
        filename
    );

    Ok(py.None())
}
