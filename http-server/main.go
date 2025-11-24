package main

import (
    "encoding/json"
    "fmt"
    "io/ioutil"
    "net/http"
)

func process(w http.ResponseWriter, req *http.Request) {
 defer req.Body.Close()
    body, _ := ioutil.ReadAll(req.Body)

    var data_in map[string]string
    err := json.Unmarshal([]byte(body), &data_in)
    if err != nil {
        // TODO: Write 404 Not found
        fmt.Println(err)
        return
    }

    fmt.Printf("Request for SNI: %v\n", data_in["name"])

    data_out := map[string]string{"server": "127.0.0.1", "port": "19443"}

    jsonData, err := json.Marshal(data_out)
    if err != nil {
      fmt.Println(err)
      return
    }

    w.Write(jsonData)
}

func main() {

    http.HandleFunc("/", process)

    http.ListenAndServe(":50001", nil)
}
