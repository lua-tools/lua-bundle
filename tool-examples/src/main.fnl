(let [zero (require :../vendor/zero)
      module (require :module)]
  (module.pony)
  (assert (= (type zero) :number)))
